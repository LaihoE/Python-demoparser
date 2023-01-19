use super::game_events::GameEvent;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::Entity;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
use crate::parsing::utils::read_file;
use crate::parsing::utils::TYPEHM;
use crate::parsing::variants::BytesVariant::Mmap3;
pub use crate::parsing::variants::*;
use crate::Parser;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

pub struct ParserState {
    pub fp: usize,
    pub tick: i32,
    pub round: i32,
    pub entities: Vec<(u32, Entity)>,
    pub stringtables: Vec<StringTable>,
    pub game_events: Vec<GameEvent>,
    pub ge_map_started_at: u64,
    pub dt_started_at: u64,
}

pub struct Maps {
    /*
    Different lookup maps used during parsing
    */
    pub serverclass_map: HashMap<u16, ServerClass, RandomState>,
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
                    serverclass_map: HashMap::default(),
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
                    dt_started_at: 0,
                    ge_map_started_at: 0,
                };
                match data {
                    Mmap3(m) => Ok(Self {
                        maps: maps,
                        settings: settings,
                        bytes: Arc::new(m),
                        state: state,
                        tasks: Vec::with_capacity(100000),
                    }),
                    BytesVariant::Vec(_) => panic!("vec"),
                }
            }
        }
    }
}
