use crate::parsing::demo_parsing::*;
use crate::parsing::utils::read_file;
use crate::parsing::utils::CACHE_ID_MAP;
use crate::parsing::variants::BytesVariant::Mmap3;
pub use crate::parsing::variants::*;
use crate::Parser;
use ahash::HashMap;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::sync::Arc;

pub struct ParserState {
    pub fp: usize,
    pub tick: i32,
    pub round: i32,
    pub entities: HashMap<i32, Entity>,
    pub stringtables: Vec<StringTable>,
    pub game_events: Vec<GameEvent>,
    pub ge_map_started_at: u64,
    pub dt_started_at: u64,
    pub workhorse: Vec<i32>,
    pub workhorse_idx: usize,
    pub workhorse_tick_start: Vec<usize>,
    pub frame_started_at: u64,
    pub test: HashMap<u32, HashMap<u32, Vec<[i32; 3]>>>,
    pub eid_cls_history: Vec<EidClsHistoryEntry>,
    pub game_event_history: Vec<GameEventHistory>,
    pub stringtable_history: Vec<StringTableHistory>,
    pub output: HashMap<i32, PropColumn>,
    pub weapon_handle_id: i32,
    pub clip_id: i32,
    pub item_def_id: i32,
    pub eid_cls_map: HashMap<i32, Vec<EidClsHistoryEntry>>,
}

pub struct Maps {
    /*
    Different lookup maps used during parsing
    */
    pub serverclass_map: HashMap<u16, ServerClass>,
    pub event_map: Option<HashMap<i32, Descriptor_t>>,
    pub dt_map: Option<HashMap<String, CSVCMsg_SendTable>>,
    pub players: BTreeMap<u64, UserInfo>,
    pub userid_sid_map: HashMap<u32, u64>,
    pub sid_entid_map: HashMap<u64, u32>,
    pub uid_eid_map: HashMap<u32, u32>,
    pub baselines: HashMap<u32, HashMap<i32, PropData>>,
    pub baseline_no_cls: HashMap<u32, Vec<u8>>,
    pub name_entid_prop: HashMap<String, usize>,
    pub name_ptype_map: HashMap<String, i32>,
    pub event_name_to_id: HashMap<String, i32>,
}

pub struct ParserSettings {
    pub only_players: bool,
    pub only_events: bool,
    pub parse_props: bool,
    pub event_name: String,
    pub parse_game_events: bool,
    pub wanted_props: Vec<String>,
    pub wanted_ticks: Vec<i32>,
    pub wanted_players: Vec<u64>,
    pub playback_frames: usize,
    pub og_names: Vec<String>,
    pub is_cache_run: bool,
    pub collect_props: Vec<String>,
}
pub struct ParserInputs {
    pub demo_path: String,
    pub parse_props: bool,
    pub only_events: bool,
    pub wanted_ticks: Vec<i32>,
    pub wanted_players: Vec<u64>,
    pub event_name: String,
    pub only_players: bool,
    pub parse_game_events: bool,
    pub wanted_props: Vec<String>,
    pub og_names: Vec<String>,
    pub collect_props: Vec<String>,
}

impl Parser {
    pub fn new(settings: ParserInputs) -> Result<Self, std::io::Error> {
        let mut extra_wanted_props = vec![];

        for p in &settings.wanted_props {
            match CACHE_ID_MAP.get(p) {
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

        match read_file(settings.demo_path) {
            Err(e) => Err(e),
            Ok(data) => {
                let maps = Maps {
                    serverclass_map: HashMap::default(),
                    event_map: Some(HashMap::default()),
                    dt_map: Some(HashMap::default()),
                    players: BTreeMap::default(),
                    userid_sid_map: HashMap::default(),
                    baselines: HashMap::default(),
                    baseline_no_cls: HashMap::default(),
                    sid_entid_map: HashMap::default(),
                    uid_eid_map: HashMap::default(),
                    name_entid_prop: HashMap::default(),
                    name_ptype_map: HashMap::default(),
                    event_name_to_id: HashMap::default(),
                };
                let mut wanted_tick_heap = BinaryHeap::new();

                for tick in &settings.wanted_ticks {
                    wanted_tick_heap.push(Reverse(*tick));
                }

                let settings = ParserSettings {
                    only_players: settings.only_players,
                    only_events: settings.only_events,
                    parse_props: settings.parse_props,
                    event_name: settings.event_name,
                    parse_game_events: settings.parse_game_events,
                    wanted_props: settings.wanted_props,
                    wanted_ticks: settings.wanted_ticks,
                    wanted_players: settings.wanted_players,
                    playback_frames: 0,
                    og_names: settings.og_names,
                    is_cache_run: false,
                    collect_props: settings.collect_props,
                };
                let state = ParserState {
                    fp: 0,
                    round: 0,
                    tick: 0,
                    entities: HashMap::default(),
                    game_events: vec![],
                    stringtables: vec![],
                    dt_started_at: 0,
                    ge_map_started_at: 0,
                    workhorse: vec![0; 20000],
                    workhorse_idx: 0,
                    workhorse_tick_start: vec![],
                    frame_started_at: 0,
                    test: HashMap::default(),
                    eid_cls_history: vec![],
                    game_event_history: vec![],
                    stringtable_history: vec![],
                    output: HashMap::default(),
                    clip_id: 0,
                    weapon_handle_id: 0,
                    item_def_id: 0,
                    eid_cls_map: HashMap::default(),
                };
                match data {
                    Mmap3(m) => Ok(Self {
                        maps: maps,
                        settings: settings,
                        bytes: Arc::new(m),
                        state: state,
                    }),
                    BytesVariant::Vec(_) => panic!("vec"),
                }
            }
        }
    }
}
