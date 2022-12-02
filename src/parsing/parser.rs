use super::game_events::GameEvent;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::parse_packet_entities;
use crate::parsing::entities::Entity;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
use crate::parsing::utils::check_round_change;
use crate::parsing::utils::read_file;
use crate::parsing::utils::TYPEHM;
use crate::parsing::variants::BytesVariant::Mmap3;
pub use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use dashmap::DashMap;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use protobuf;
use protobuf::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::slice;
use std::sync::Arc;
use std::sync::RwLock;
use std::u8;
use threadpool::ThreadPool;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub struct Parser {
    pub maps: Maps,
    pub settings: ParserSettings,
    pub state: ParserState,
    pub bytes: Arc<Mmap>,
    // General purpose int vec, for perf reasons
    pub pool: ThreadPool,
}

pub struct ParserState {
    pub fp: usize,
    pub tick: i32,
    pub round: i32,
    pub entities: Vec<(u32, Entity)>,
    pub stringtables: Vec<StringTable>,
    pub game_events: Vec<GameEvent>,
}

pub struct Maps {
    /*
    Different lookup maps used during parsing
    */
    pub serverclass_map: Arc<RwLock<HashMap<u16, ServerClass, RandomState>>>,
    pub event_map: Option<HashMap<i32, Descriptor_t, RandomState>>,
    pub dt_map: Option<HashMap<String, CSVCMsg_SendTable, RandomState>>,
    pub players: HashMap<u64, UserInfo, RandomState>,
    pub userid_sid_map: HashMap<u32, Vec<(u64, i32)>, RandomState>,
    pub sid_entid_map: HashMap<u64, Vec<(u32, i32)>>,
    pub uid_eid_map: HashMap<u32, Vec<(u32, i32)>, RandomState>,
    pub baselines: HashMap<u32, HashMap<String, PropData>>,
    pub baseline_no_cls: HashMap<u32, Vec<u8>>,
}

pub struct ParserSettings {
    pub only_players: bool,
    pub only_header: bool,
    pub parse_props: bool,
    pub event_name: String,
    pub no_gameevents: bool,
    pub early_exit_tick: i32,
    pub wanted_props: Vec<String>,
    pub wanted_ticks: HashSet<i32, RandomState>,
    pub wanted_players: Vec<u64>,
    pub playback_frames: usize,
    pub og_names: Vec<String>,
}

pub struct MsgBluePrint {
    pub msg: u32,
    // Byte idx where msg start and end
    pub start_idx: usize,
    pub end_inx: usize,
    pub tick: i32,
}

impl Parser {
    pub fn new(
        demo_path: String,
        parse_props: bool,
        wanted_ticks: Vec<i32>,
        wanted_players: Vec<u64>,
        mut wanted_props: Vec<String>,
        event_name: String,
        only_players: bool,
        only_header: bool,
        no_gameevents: bool,
        early_exit_tick: i32,
        og_names: Vec<String>,
    ) -> Result<Self, std::io::Error> {
        let mut extra_wanted_props = vec![];
        for p in &wanted_props {
            match TYPEHM.get(p) {
                Some(_) => match &p[(p.len() - 1)..] {
                    "X" => extra_wanted_props.push((&p[..p.len() - 2]).to_owned()),
                    "Y" => extra_wanted_props.push((&p[..p.len() - 2]).to_owned()),
                    "Z" => extra_wanted_props.push((&p[..p.len() - 2]).to_owned()),
                    _ => {}
                },
                None => {
                    panic!("Prop: {} not found", p);
                }
            }
        }

        wanted_props.extend(extra_wanted_props);
        match read_file(demo_path) {
            Err(e) => Err(e),
            Ok(data) => {
                let maps = Maps {
                    serverclass_map: Arc::new(RwLock::new(HashMap::default())),
                    event_map: Some(HashMap::default()),
                    dt_map: Some(HashMap::default()),
                    players: HashMap::default(),
                    userid_sid_map: HashMap::default(),
                    baselines: HashMap::default(),
                    baseline_no_cls: HashMap::default(),
                    sid_entid_map: HashMap::default(),
                    uid_eid_map: HashMap::default(),
                };
                let settings = ParserSettings {
                    only_players: only_players,
                    only_header: only_header,
                    parse_props: parse_props,
                    event_name: event_name,
                    no_gameevents: no_gameevents,
                    early_exit_tick: early_exit_tick,
                    wanted_props: wanted_props,
                    wanted_ticks: HashSet::from_iter(wanted_ticks.iter().cloned()),
                    wanted_players: wanted_players,
                    playback_frames: 0,
                    og_names: og_names,
                };
                let state = ParserState {
                    fp: 0,
                    round: 0,
                    tick: 0,
                    entities: vec![],
                    game_events: vec![],
                    stringtables: vec![],
                };
                match data {
                    Mmap3(m) => Ok(Self {
                        maps: maps,
                        settings: settings,
                        bytes: Arc::new(m),
                        state: state,
                        pool: ThreadPool::new(12),
                    }),
                    BytesVariant::Vec(_) => panic!("vec"),
                }
            }
        }
    }
}

impl Parser {
    pub fn start_parsing(
        &mut self,
        props_names: &Vec<String>,
    ) -> Arc<DashMap<String, HashMap<u32, VarVec>>> {
        let mut ticks_props: HashMap<String, PropColumn, RandomState> = HashMap::default();
        let mut data: Arc<DashMap<String, HashMap<u32, VarVec>>> = Arc::new(DashMap::default());

        for _ in 0..10000 {
            self.state.entities.push((
                1111111,
                Entity {
                    class_id: 0,
                    entity_id: 1111111,
                    props: HashMap::default(),
                },
            ));
        }

        while self.state.fp < self.bytes.len() as usize {
            let (cmd, tick) = self.read_frame();
            if tick > self.settings.early_exit_tick {
                break;
            }
            self.state.tick = tick;
            // EARLY EXIT
            if self.settings.only_header {
                break;
            }

            self.parse_cmd(cmd, data.clone());
        }
        //self.pool.j(data)
        self.pool.join();
        data
    }

    #[inline(always)]
    pub fn parse_cmd(&mut self, cmd: u8, data: Arc<DashMap<String, HashMap<u32, VarVec>>>) {
        match cmd {
            1 => self.parse_packet(data),
            2 => self.parse_packet(data),
            6 => self.parse_datatable(),
            _ => {}
        }
    }

    #[inline(always)]
    pub fn parse_packet(&mut self, out_data: Arc<DashMap<String, HashMap<u32, VarVec>>>) {
        check_round_change(&self.state.entities, &mut self.state.round);
        self.state.fp += 160;
        let packet_len = self.read_i32();
        let goal_inx = self.state.fp + packet_len as usize;

        while self.state.fp < goal_inx {
            let msg = self.read_varint();
            let size = self.read_varint();

            // Get byte boundaries for this msg, threads will then read the bytes
            let before_inx = self.state.fp.clone();
            self.skip_n_bytes(size);
            let after_inx = self.state.fp.clone();
            // Information needed to parse a msg, passed to threads as a "job"
            let msg_blueprint = MsgBluePrint {
                msg: msg,
                start_idx: before_inx,
                end_inx: after_inx,
                tick: self.state.tick,
            };
            self.msg_handler(msg_blueprint, out_data.clone());
        }
    }
    pub fn msg_handler(
        &self,
        blueprint: MsgBluePrint,
        out_data: Arc<DashMap<String, HashMap<u32, VarVec>>>,
    ) {
        match blueprint.msg {
            //25 => Parser::parse_game_events(game_event, mmap),
            //30 => Parser::game_event_list(game_event, mmap),
            26 => {
                let bc = self.bytes.clone();
                let svc_clone = self.maps.serverclass_map.clone();

                self.pool
                    .execute(move || parse_packet_entities(blueprint, bc, svc_clone, out_data))
            }
            _ => {}
        }

        /*
        match msg as i32 {
            // Game event
            25 => {
                if !no_gameevents {
                    let game_event = Message::parse_from_bytes(data);
                    match game_event {
                        Ok(ge) => {
                            let game_event = ge;
                            let (game_events, con_tick) = self.parse_game_events(game_event);
                            is_con_tick = con_tick;
                            self.state.game_events.extend(game_events);
                        }
                        Err(e) => panic!(
                            "Failed to parse game event at tick {}. Error: {e}",
                            self.state.tick
                        ),
                    }
                }
            }
            // Game event list
            30 => {
                if !no_gameevents {
                    let event_list = Message::parse_from_bytes(data);
                    match event_list {
                        Ok(ev) => {
                            let event_list = ev;
                            self.parse_game_event_map(event_list)
                        }
                        Err(e) => panic!(
                            "Failed to parse game event LIST at tick {}. Error: {e}",
                            self.state.tick
                        ),
                    }
                }
            }
            // Packet entites
            26 => {
                if parse_props {
                    let pack_ents = Message::parse_from_bytes(data);
                    match pack_ents {
                        Ok(pe) => {
                            let pack_ents = pe;

                            let svc_clone = self.maps.serverclass_map.clone();
                            let out_clone = out_data.clone();

                            self.pool.execute(move || {
                                Parser::parse_packet_entities(
                                    pack_ents,
                                    svc_clone,
                                    out_clone,
                                    tick.clone(),
                                );
                            });
                        }
                        Err(e) => panic!(
                            "Failed to parse Packet entities at tick {}. Error: {e}",
                            self.state.tick
                        ),
                    }
                }
            }
            // Create string table
            12 => {
                let string_table = Message::parse_from_bytes(data);
                match string_table {
                    Ok(st) => {
                        let string_table = st;
                        self.create_string_table(string_table);
                    }
                    Err(e) => panic!(
                        "Failed to parse String table at tick {}. Error: {e}",
                        self.state.tick
                    ),
                }
            }
            // Update string table
            13 => {
                let data = Message::parse_from_bytes(data);
                match data {
                    Ok(st) => {
                        let data = st;
                        self.update_string_table_msg(data);
                    }
                    Err(e) => panic!(
                        "Failed to parse String table at tick {}. Error: {e}",
                        self.state.tick
                    ),
                }
            }
            _ => {}
        }
        */
    }
}
