use super::game_events::GameEvent;
use super::read_bits::PropData;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::Entity;
use crate::parsing::extract_props::extract_props;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use flate2::read::GzDecoder;
use fxhash::FxHashMap;
use hashbrown::HashMap;
use hashbrown::HashSet;
use protobuf;
use protobuf::Message;
use std::io::Read;
use std::path::Path;

#[allow(dead_code)]
pub struct Frame {
    pub cmd: u8,
    pub tick: i32,
    pub playerslot: u8,
}
#[derive(Debug, Clone)]
pub enum VarVec {
    U64(Vec<u64>),
    F32(Vec<f32>),
    I64(Vec<i64>),
    String(Vec<String>),
}
#[derive(Debug, Clone)]
pub struct PropColumn {
    pub dtype: String,
    pub data: VarVec,
}

pub struct Demo {
    pub fp: usize,
    pub tick: i32,
    pub cmd: u8,
    pub bytes: Vec<u8>,
    pub class_bits: u32,
    pub event_list: Option<CSVCMsg_GameEventList>,
    pub event_map: Option<HashMap<i32, Descriptor_t>>,
    pub dt_map: Option<HashMap<String, CSVCMsg_SendTable>>,
    pub serverclass_map: HashMap<u16, ServerClass>,
    pub entities: Option<HashMap<u32, Option<Entity>>>,
    pub bad: Vec<String>,
    pub stringtables: Vec<StringTable>,
    pub players: Vec<UserInfo>,
    pub parse_props: bool,
    pub game_events: Vec<GameEvent>,
    pub event_name: String,
    pub cnt: i32,
    pub wanted_props: Vec<String>,
    pub wanted_ticks: HashSet<i32>,
    pub wanted_players: Vec<u64>,
    pub round: i32,
    pub players_connected: i32,
    pub only_players: bool,
    pub only_header: bool,
}

impl VarVec {
    pub fn push_propdata(&mut self, item: PropData) {
        match item {
            PropData::F32(p) => match self {
                VarVec::F32(f) => f.push(p),
                _ => {}
            },
            PropData::I64(p) => match self {
                VarVec::I64(f) => f.push(p),
                _ => {}
            },
            PropData::String(p) => match self {
                VarVec::String(f) => f.push(p),
                _ => {}
            },
            _ => panic!("bad type for prop"),
        }
    }
    pub fn push_string(&mut self, data: String) {
        match self {
            VarVec::String(f) => f.push(data),
            _ => {}
        }
    }
}

impl Demo {
    pub fn decompress_gz(bytes: Vec<u8>) -> Vec<u8> {
        let mut gz = GzDecoder::new(&bytes[..]);
        let mut out: Vec<u8> = vec![];
        gz.read_to_end(&mut out).unwrap();
        out
    }

    pub fn read_file(demo_path: String) -> Result<Vec<u8>, std::io::Error> {
        let result = std::fs::read(&demo_path);
        match result {
            // FILE COULD NOT BE READ
            Err(e) => Err(e), //panic!("The demo could not be found. Error: {}", e),
            Ok(bytes) => {
                let extension = Path::new(&demo_path).extension().unwrap();
                match extension.to_str().unwrap() {
                    "gz" => Ok(Demo::decompress_gz(bytes)),
                    _ => Ok(bytes),
                }
            }
        }
    }

    pub fn new(
        demo_path: String,
        parse_props: bool,
        wanted_ticks: Vec<i32>,
        wanted_players: Vec<u64>,
        wanted_props: Vec<String>,
        event_name: String,
        only_players: bool,
        only_header: bool,
    ) -> Result<Self, std::io::Error> {
        let bytes = Demo::read_file(demo_path);
        match bytes {
            Ok(bytes) => {
                Ok(Self {
                    bytes: bytes,
                    fp: 0,
                    cmd: 0,
                    tick: 0,
                    cnt: 0,
                    round: 0,
                    event_list: None,
                    event_map: None,
                    class_bits: 0,
                    parse_props: parse_props,
                    event_name: event_name,
                    bad: Vec::new(),
                    dt_map: Some(HashMap::default()),
                    serverclass_map: HashMap::default(),
                    entities: Some(HashMap::default()),
                    stringtables: Vec::new(),
                    players: Vec::new(),
                    // changing ones
                    wanted_props: wanted_props,
                    game_events: Vec::new(),
                    wanted_players: wanted_players,
                    wanted_ticks: HashSet::from_iter(wanted_ticks),
                    players_connected: 0,
                    only_header: only_header,
                    only_players: only_players,
                })
            }
            Err(e) => Err(e),
        }
    }
}

impl Demo {
    pub fn parse_frame(&mut self, props_names: &Vec<String>) -> FxHashMap<String, PropColumn> {
        // Main loop
        let mut ticks_props: FxHashMap<String, PropColumn> = FxHashMap::default();

        while self.fp < self.bytes.len() as usize {
            let f = self.read_frame_bytes();
            self.tick = f.tick;

            // EARLY EXITS
            if self.only_players {
                if Demo::all_players_connected(self.players_connected) {
                    break;
                }
            }
            if self.only_header {
                if Demo::all_players_connected(self.players_connected) {
                    break;
                }
            }
            for player in &self.players {
                if self.wanted_ticks.contains(&self.tick) || self.wanted_ticks.len() == 0 {
                    if self.wanted_players.contains(&player.xuid) || self.wanted_players.len() == 0
                    {
                        if self
                            .entities
                            .as_ref()
                            .unwrap()
                            .contains_key(&player.entity_id)
                        {
                            if self.entities.as_ref().unwrap()[&player.entity_id].is_some() {
                                let ent = self.entities.as_ref().unwrap()[&player.entity_id]
                                    .as_ref()
                                    .unwrap();

                                for prop_name in props_names {
                                    // println!("{:?}", ent.props);
                                    match ent.props.get(prop_name) {
                                        None => {}
                                        Some(e) => {
                                            //println!("{} {:?}", prop_name, e.data);
                                            ticks_props
                                                .entry(e.prop_name.to_string())
                                                .or_insert_with(|| PropColumn {
                                                    dtype: "f32".to_string(),
                                                    data: VarVec::F32(Vec::new()),
                                                })
                                                .data
                                                .push_propdata(e.data.clone());
                                            // EXTRA
                                            ticks_props
                                                .entry("tick".to_string())
                                                .or_insert_with(|| PropColumn {
                                                    dtype: "i32".to_string(),
                                                    data: VarVec::String(vec![]),
                                                })
                                                .data
                                                .push_string(self.tick.to_string());

                                            ticks_props
                                                .entry("steamid".to_string())
                                                .or_insert_with(|| PropColumn {
                                                    dtype: "u64".to_string(),
                                                    data: VarVec::String(vec![]),
                                                })
                                                .data
                                                .push_string(player.xuid.to_string());
                                            ticks_props
                                                .entry("name".to_string())
                                                .or_insert_with(|| PropColumn {
                                                    dtype: "u64".to_string(),
                                                    data: VarVec::String(vec![]),
                                                })
                                                .data
                                                .push_string(player.name.to_string());
                                        }
                                    }
                                }
                                /*


                                */
                                //tick_props.push(("name".to_string(), wanted_name));
                            }
                        }
                    }
                }
            }

            self.parse_cmd(f.cmd);
        }
        ticks_props
    }

    pub fn parse_cmd(&mut self, cmd: u8) {
        match cmd {
            1 => self.parse_packet(),
            2 => self.parse_packet(),
            6 => self.parse_datatable(),
            _ => {
                //println!("CMD {}", cmd); //panic!("UNK CMD")
            } //,
        }
    }

    pub fn all_players_connected(total_connected: i32) -> bool {
        if total_connected == 10 {
            return true;
        }
        return false;
    }

    pub fn parse_packet(&mut self) {
        self.fp += 160;
        let packet_len = self.read_i32();
        let goal_inx = self.fp + packet_len as usize;
        let parse_props = self.parse_props;
        while self.fp < goal_inx {
            let msg = self.read_varint();
            let size = self.read_varint();
            let data = self.read_n_bytes(size);

            match msg as i32 {
                // Game event
                25 => {
                    let game_event = Message::parse_from_bytes(&data);
                    match game_event {
                        Ok(ge) => {
                            let game_event = ge;
                            let game_events = self.parse_game_events(game_event);
                            self.game_events.extend(game_events);
                        }
                        Err(e) => panic!(
                            "Failed to parse game event at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                // Game event list
                30 => {
                    let event_list = Message::parse_from_bytes(&data);
                    match event_list {
                        Ok(ev) => {
                            let event_list = ev;
                            self.parse_game_event_map(event_list)
                        }
                        Err(e) => panic!(
                            "Failed to parse game event LIST at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                // Packet entites
                26 => {
                    if parse_props {
                        let pack_ents = Message::parse_from_bytes(&data);
                        match pack_ents {
                            Ok(pe) => {
                                let pack_ents = pe;
                                self.parse_packet_entities(pack_ents, parse_props);
                            }
                            Err(e) => panic!(
                                "Failed to parse Packet entities at tick {}. Error: {e}",
                                self.tick
                            ),
                        }
                    }
                }
                // Create string table
                12 => {
                    let string_table = Message::parse_from_bytes(&data);
                    match string_table {
                        Ok(st) => {
                            let string_table = st;
                            self.create_string_table(string_table);
                        }
                        Err(e) => panic!(
                            "Failed to parse String table at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                // Update string table
                13 => {
                    let data = Message::parse_from_bytes(&data);
                    match data {
                        Ok(st) => {
                            let data = st;
                            self.update_string_table_msg(data);
                        }
                        Err(e) => panic!(
                            "Failed to parse String table at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                _ => {}
            }
        }
    }
}
