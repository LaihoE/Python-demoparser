use super::parser::MsgBluePrint;
use super::stringtables::UserInfo;
use crate::parsing::entities::Entity;
use crate::parsing::parser::Parser;
use crate::parsing::parser::*;
use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event::Key_t;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_GameEvent;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use memmap2::Mmap;
use protobuf::Message;
use pyo3::prelude::*;
use std::collections::HashMap;

fn parse_key(key: &Key_t) -> Option<KeyData> {
    match key.type_() {
        1 => Some(KeyData::Str(key.val_string().to_owned())),
        2 => Some(KeyData::Float(key.val_float())),
        3 => Some(KeyData::Long(key.val_long())),
        4 => Some(KeyData::Short(key.val_short().try_into().unwrap())),
        5 => Some(KeyData::Byte(key.val_byte().try_into().unwrap())),
        6 => Some(KeyData::Bool(key.val_bool())),
        7 => Some(KeyData::Uint64(key.val_uint64())),
        _ => panic!("Unkown key type for game event key"),
    }
}
#[derive(Debug)]
pub enum KeyData {
    Str(String),
    Float(f32),
    Long(i32),
    Short(i16),
    Byte(u8),
    Bool(bool),
    Uint64(u64),
}
impl Default for KeyData {
    fn default() -> Self {
        KeyData::Bool(false)
    }
}

impl KeyData {
    pub fn from_pdata(pdata: &PropData) -> Self {
        match pdata {
            PropData::F32(f) => KeyData::Float(*f),
            PropData::I32(f) => KeyData::Long(*f),
            PropData::String(f) => KeyData::Str(f.to_string()),
            _ => panic!("not yet suppored"),
        }
    }
    pub fn to_string_py(&self, py: Python<'_>) -> PyObject {
        match self {
            KeyData::Str(f) => f.to_string().to_object(py),
            KeyData::Float(f) => f.to_object(py),
            KeyData::Long(f) => f.to_object(py),
            KeyData::Short(f) => f.to_object(py),
            KeyData::Byte(f) => f.to_object(py),
            KeyData::Bool(f) => f.to_object(py),
            KeyData::Uint64(f) => f.to_object(py),
        }
    }
}

#[derive(Debug)]
pub struct NameDataPair {
    pub name: String,
    pub data: Option<KeyData>,
}
#[derive(Debug)]
pub struct GameEvent {
    pub name: String,
    pub fields: Vec<NameDataPair>,
}

impl GameEvent {
    pub fn to_py_tuples(&self, py: Python<'_>) -> Vec<(String, PyObject)> {
        let mut py_tuples: Vec<(String, PyObject)> = Vec::new();
        for pair in &self.fields {
            let name = &pair.name;
            let val = match &pair.data {
                Some(d) => d.to_string_py(py),
                None => "None".to_object(py),
            };
            //let val = pair.data.to_string_py(py);
            py_tuples.push((name.to_string(), val));
        }
        py_tuples
    }
}

pub fn gen_name_val_pairs(
    game_event: &CSVCMsg_GameEvent,
    event: &Descriptor_t,
    tick: &i32,
    //uid_eid_map: &HashMap<u32, u64, RandomState>,
) -> Vec<NameDataPair> {
    // Takes the msg and its descriptor and parses (name, val) pairs from it
    let mut kv_pairs: Vec<NameDataPair> = Vec::new();

    for i in 0..game_event.keys.len() {
        let ge = &game_event.keys[i];
        let desc = &event.keys[i];
        let val = parse_key(ge);
        kv_pairs.push(NameDataPair {
            name: desc.name().to_owned(),
            data: val,
        })
    }
    kv_pairs.push(NameDataPair {
        name: "tick".to_owned(),
        data: Some(KeyData::Long(*tick)),
    });
    kv_pairs
}

impl Parser {
    pub fn parse_game_events(
        blueprint: &MsgBluePrint,
        mmap: &Mmap,
        game_events_map: &HashMap<i32, Descriptor_t, RandomState>,
        wanted_event: &str,
    ) -> JobResult {
        let wanted_bytes = &mmap[blueprint.start_idx..blueprint.end_idx];
        let msg: CSVCMsg_GameEvent = Message::parse_from_bytes(wanted_bytes).unwrap();
        let mut game_events: Vec<GameEvent> = Vec::new();
        let event_desc = &game_events_map[&msg.eventid()];

        if event_desc.name() == wanted_event {
            let name_data_pairs = gen_name_val_pairs(&msg, event_desc, &blueprint.tick);

            game_events.push({
                GameEvent {
                    name: event_desc.name().to_owned(),
                    fields: name_data_pairs,
                }
            });
        }
        JobResult::GameEvents(game_events)
    }

    pub fn parse_game_event_map(&mut self, blueprint: &MsgBluePrint) {
        let wanted_bytes = &self.bytes[blueprint.start_idx..blueprint.end_idx];
        let msg: CSVCMsg_GameEventList = Message::parse_from_bytes(wanted_bytes).unwrap();
        let mut hm: HashMap<i32, Descriptor_t, RandomState> = HashMap::default();
        for event_desc in msg.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        self.maps.event_map = Some(hm);
    }
}
