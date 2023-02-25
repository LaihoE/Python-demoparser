use crate::parsing::demo_parsing::ByteReader;
use crate::parsing::parser::Parser;
use crate::parsing::parser::*;
use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event::Key_t;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_GameEvent;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use derive_more::TryInto;
use memmap2::Mmap;
use protobuf::Message;
use pyo3::prelude::*;
use std::collections::HashMap;

pub struct GameEventHistory {
    pub byte: i32,
    pub id: i32,
}

impl Parser {
    pub fn parse_game_event(&mut self, byte_reader: &mut ByteReader, size: usize) {
        let wanted_bytes = &self.bytes[byte_reader.byte_idx..byte_reader.byte_idx + size as usize];
        byte_reader.skip_n_bytes(size.try_into().unwrap());
        let game_event: CSVCMsg_GameEvent = Message::parse_from_bytes(wanted_bytes).unwrap();
        self.state.game_event_history.push(GameEventHistory {
            byte: self.state.frame_started_at as i32,
            id: game_event.eventid(),
        });
        let event_desc = &self.maps.event_map.as_ref().unwrap()[&game_event.eventid()];
        let name_data_pairs = gen_name_val_pairs(&game_event, event_desc, &self.state.tick.clone());
    }

    pub fn parse_game_event_map(&mut self, byte_reader: &mut ByteReader, size: usize) {
        self.state.ge_map_started_at = self.state.frame_started_at;
        let wanted_bytes = &self.bytes[byte_reader.byte_idx..byte_reader.byte_idx + size as usize];
        byte_reader.skip_n_bytes(size.try_into().unwrap());
        let game_event_list: CSVCMsg_GameEventList =
            Message::parse_from_bytes(wanted_bytes).unwrap();

        let mut hm: HashMap<i32, Descriptor_t, RandomState> = HashMap::default();
        for event_desc in game_event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        self.maps.event_map = Some(hm);
    }
}

fn parse_key(key: &Key_t) -> KeyData {
    match key.type_() {
        1 => KeyData::Str(key.val_string().to_owned()),
        2 => KeyData::Float(key.val_float()),
        3 => KeyData::Long(key.val_long()),
        4 => KeyData::Short(key.val_short().try_into().unwrap()),
        5 => KeyData::Byte(key.val_byte().try_into().unwrap()),
        6 => KeyData::Bool(key.val_bool()),
        7 => KeyData::Uint64(key.val_uint64()),
        _ => panic!("Unknown key type for game event key"),
    }
}

#[derive(Debug, Clone, TryInto)]
#[try_into(owned, ref)]
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

impl TryInto<i64> for KeyData {
    type Error = ();

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            Self::Long(l) => Ok(l as i64),
            Self::Byte(b) => Ok(b as i64),
            Self::Short(s) => Ok(s as i64),
            _ => Err(()),
        }
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

#[derive(Debug, Clone)]
pub struct NameDataPair {
    pub name: String,
    pub data: KeyData,
    pub data_type: i32,
}
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub name: String,
    pub fields: Vec<NameDataPair>,
    pub tick: i32,
    pub byte: usize,
    pub id: i32,
}

pub fn gen_name_val_pairs(
    game_event: &CSVCMsg_GameEvent,
    event: &Descriptor_t,
    tick: &i32,
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
            data_type: ge.type_(),
        })
    }
    kv_pairs.push(NameDataPair {
        name: "tick".to_owned(),
        data: KeyData::Long(*tick),
        data_type: 3,
    });
    kv_pairs
}
