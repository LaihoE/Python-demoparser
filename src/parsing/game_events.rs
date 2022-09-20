use std::any::Any;

use crate::Demo;
use csgoproto::netmessages::csvcmsg_game_event::Key_t;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_CreateStringTable;
use csgoproto::netmessages::CSVCMsg_GameEvent;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use hashbrown::HashMap;
use protobuf::Message;
use pyo3::prelude::*;

use super::stringtables::UserInfo;

#[derive(Debug, Default)]
pub struct HurtEvent {
    pub userid: i32,
    pub attacker: i32,
    pub health: i32,
    pub armor: i32,
    pub weapon: String,
    pub dmg_health: i32,
    pub dmg_armor: i32,
    pub hitgroup: i32,
}

fn parse_key(key: &Key_t) -> KeyData {
    match key.type_() {
        1 => return KeyData::StrData(key.val_string().to_owned()),
        2 => return KeyData::FloatData(key.val_float()),
        3 => return KeyData::LongData(key.val_long()),
        4 => return KeyData::ShortData(key.val_short().try_into().unwrap()),
        5 => return KeyData::ByteData(key.val_byte().try_into().unwrap()),
        6 => return KeyData::BoolData(key.val_bool()),
        7 => return KeyData::Uint64Data(key.val_uint64()),
        _ => panic!("KEYDATA FAILED"),
    }
}

fn parse_key_steamid(key: &Key_t, players: &Vec<UserInfo>) -> KeyData {
    let mut ent_id = key.val_short();
    for player in players {
        if player.entity_id as i32 == ent_id {
            let xuid: u64 = player.xuid.try_into().unwrap();
            match key.type_() {
                4 => return KeyData::StrData(xuid.to_string()),
                _ => panic!("KEYDATA FAILED"),
            }
        }
    }
    match key.type_() {
        4 => return KeyData::StrData(key.val_short().to_string()),
        _ => panic!("KEYDATA FAILED"),
    }
}

#[derive(Debug)]
pub enum KeyData {
    StrData(String),
    FloatData(f32),
    LongData(i32),
    ShortData(i16),
    ByteData(u8),
    BoolData(bool),
    Uint64Data(u64),
}
impl Default for KeyData {
    fn default() -> Self {
        KeyData::BoolData(false)
    }
}
impl KeyData {
    pub fn to_string_py(&self) -> String {
        match self {
            KeyData::StrData(f) => f.to_string(),
            KeyData::FloatData(f) => f.to_string(),
            KeyData::LongData(f) => f.to_string(),
            KeyData::ShortData(f) => f.to_string(),
            KeyData::ByteData(f) => f.to_string(),
            KeyData::BoolData(f) => f.to_string(),
            KeyData::Uint64Data(f) => f.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct NameDataPair {
    pub name: String,
    pub data: KeyData,
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
            let val = pair.data.to_string_py().to_object(py);
            py_tuples.push((name.to_string(), val));
        }
        py_tuples
    }
}

pub fn gen_name_val_pairs(
    game_event: &CSVCMsg_GameEvent,
    event: &Descriptor_t,
    tick: &i32,
    players: &Vec<UserInfo>,
) -> Vec<NameDataPair> {
    // Takes the msg and its descriptor and parses (name, val) pairs from it
    let mut kv_pairs: Vec<NameDataPair> = Vec::new();

    for i in 0..game_event.keys.len() {
        let ge = &game_event.keys[i];
        let desc = &event.keys[i];

        let mut val = parse_key(ge);

        if (desc.name() == "userid" || desc.name() == "attacker") && ge.type_() == 4 {
            val = parse_key_steamid(ge, players);
        }

        kv_pairs.push(NameDataPair {
            name: desc.name().to_owned(),
            data: val,
        })
    }
    kv_pairs.push(NameDataPair {
        name: "tick".to_owned(),
        data: KeyData::LongData(*tick),
    });
    kv_pairs
}

pub fn _match_data_to_game_event(
    namedatavec: &Vec<NameDataPair>,
    event_name: &str,
    wanted_event: &String,
) -> bool {
    if event_name.contains(wanted_event) {
        return true;
    } else {
        return false;
    }
}

impl Demo {
    pub fn parse_game_events(
        &self,
        game_event: CSVCMsg_GameEvent,
        players: &Vec<UserInfo>,
    ) -> Vec<GameEvent> {
        let mut game_events: Vec<GameEvent> = Vec::new();

        let event_desc = &self.event_map;
        match event_desc {
            Some(ev_desc_map) => {
                let event_desc = &ev_desc_map[&game_event.eventid()];
                let name_data_pairs =
                    gen_name_val_pairs(&game_event, &event_desc, &self.tick, players);
                if _match_data_to_game_event(&name_data_pairs, event_desc.name(), &self.event_name)
                {
                    game_events.push({
                        GameEvent {
                            name: event_desc.name().to_owned(),
                            fields: name_data_pairs,
                        }
                    })
                }
            }
            None => {
                panic!("Game event was not found in envent list passed earlier");
            }
        }

        game_events
    }
    pub fn parse_game_event_list(&mut self, event_list: CSVCMsg_GameEventList) {
        let mut hm: HashMap<i32, Descriptor_t> = HashMap::default();

        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        self.event_map = Some(hm);
    }
}
