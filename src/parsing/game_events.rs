use crate::parsing::entities::Entity;
use super::stringtables::UserInfo;
use crate::parsing::variants::*;
use crate::Demo;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event::Key_t;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_GameEvent;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use pyo3::prelude::*;
use std::collections::HashMap;

/*
All of these are relatively cheap operations, doesn't really matter how performant
*/


fn parse_key(key: &Key_t) -> Option<KeyData> {
    match key.type_() {
        1 => return Some(KeyData::StrData(key.val_string().to_owned())),
        2 => return Some(KeyData::FloatData(key.val_float())),
        3 => return Some(KeyData::LongData(key.val_long())),
        4 => return Some(KeyData::ShortData(key.val_short().try_into().unwrap())),
        5 => return Some(KeyData::ByteData(key.val_byte().try_into().unwrap())),
        6 => return Some(KeyData::BoolData(key.val_bool())),
        7 => return Some(KeyData::Uint64Data(key.val_uint64())),
        _ => panic!("Unkown key type for game event key"),
    }
}

fn parse_key_steamid(key: &Key_t, uid_sid_map: &HashMap<u32, u64, RandomState>) -> KeyData {
    let user_id = key.val_short();

    match uid_sid_map.get(&(user_id as u32)) {
        None => KeyData::LongData(0), //panic!("USERID: {} not found in mapping to steamid", user_id),
        Some(u) => {
            return KeyData::Uint64Data(*u);
        }
    }
}

pub fn parse_props(key: &Key_t, entities: &Vec<(u32, Entity)>, wanted_props: &Vec<String>, prefix: String) -> Vec<NameDataPair>
{
    let ent_id = key.val_short();
    let mut all_pairs = vec![];
    match entities.get(ent_id as usize) {
        None => all_pairs,
        Some(ent) =>{
            for wanted_prop in wanted_props{
                match ent.1.props.get(wanted_prop){
                    Some(p) => {
                        match &p.data {
                            PropData::F32(f) => {
                                all_pairs.push(NameDataPair{
                                    name: prefix.clone() + &p.prop_name,
                                    data: Some(KeyData::FloatData(*f)),
                                });
                            }
                            PropData::I32(i) => {
                                all_pairs.push(NameDataPair{
                                    name: prefix.clone() + &p.prop_name,
                                    data: Some(KeyData::LongData(*i)),
                                });
                            }
                            PropData::String(s) => {
                                all_pairs.push(NameDataPair{
                                    name: p.prop_name.to_owned(),
                                    data: Some(KeyData::StrData(s.clone())),
                                });
                            }
                            PropData::VecXY(_) => {
                                // Handled by above sub-types and lets not create it here
                            }
                            
                        _ => {
                            all_pairs.push(NameDataPair{
                                name: prefix.clone() + &p.prop_name,
                                data: None,
                            });
                        }
                        }
                    },
                    None => { 
                        all_pairs.push(NameDataPair{
                        name: prefix.clone() + &wanted_prop,
                        data: None,
                        });
                    },
                };
                
            }
            all_pairs
        }
    }
}

fn parse_key_steam_name(
    key: &Key_t,
    players: &HashMap<u64, UserInfo, RandomState>,
    uid_sid_map: &HashMap<u32, u64, RandomState>,
) -> KeyData {
    let uid = key.val_short();
    match uid_sid_map.get(&(uid as u32)) {
        None => return KeyData::StrData("None".to_string()),
        Some(sid) => {
            for player in players.values() {
                if &player.xuid == sid {
                    match key.type_() {
                        4 => {
                            return KeyData::StrData(
                                player
                                    .name
                                    .to_string()
                                    .trim_matches(char::from(0))
                                    .to_string(),
                            )
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    return KeyData::StrData("None".to_string());
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
    pub fn to_string_py(&self, py: Python<'_>) -> PyObject {
        match self {
            KeyData::StrData(f) => f.to_string().to_object(py),
            KeyData::FloatData(f) => f.to_object(py),
            KeyData::LongData(f) => f.to_object(py),
            KeyData::ShortData(f) => f.to_object(py),
            KeyData::ByteData(f) => f.to_object(py),
            KeyData::BoolData(f) => f.to_object(py),
            KeyData::Uint64Data(f) => f.to_object(py),
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
                Some(d) => {
                    d.to_string_py(py)
                }
                None => {
                    "None".to_object(py)
                }
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
    uid_sid_map: &HashMap<u32, u64, RandomState>,
    players: &HashMap<u64, UserInfo, RandomState>,
    round: i32,
    entities: &Vec<(u32, Entity)>,
    wanted_props: &Vec<String>,
) -> Vec<NameDataPair> {
    // Takes the msg and its descriptor and parses (name, val) pairs from it
    let mut kv_pairs: Vec<NameDataPair> = Vec::new();

    for i in 0..game_event.keys.len() {
        let ge = &game_event.keys[i];
        let desc = &event.keys[i];

        match desc.name() {
            "attacker" => {
                let steamid = parse_key_steamid(ge, uid_sid_map);
                kv_pairs.push(NameDataPair {
                    name: "player_steamid".to_string(),
                    data: Some(steamid),
                });
                let steam_name = parse_key_steam_name(ge, &players, uid_sid_map);
                kv_pairs.push(NameDataPair {
                    name: "player_name".to_string(),
                    data: Some(steam_name),
                });
                let props = parse_props(ge, entities, wanted_props, "player_".to_string());
                for p in props{
                    kv_pairs.push(p);
                }
            }
            "userid" => {
                let steamid = parse_key_steamid(ge, uid_sid_map);
                kv_pairs.push(NameDataPair {
                    name: "attacker_steamid".to_string(),
                    data: Some(steamid),
                });
                let steam_name = parse_key_steam_name(ge, &players, uid_sid_map);
                kv_pairs.push(NameDataPair {
                    name: "attacker_name".to_string(),
                    data: Some(steam_name),
                });
                let props = parse_props(ge, entities, wanted_props, "attacker_".to_string());
                for p in props{
                    kv_pairs.push(p);
                }
            }
            _ => {
                let val = parse_key(ge);
                kv_pairs.push(NameDataPair {
                    name: desc.name().to_owned(),
                    data: val,
                })
            }
        }
    }
    kv_pairs.push(NameDataPair {
        name: "tick".to_owned(),
        data: Some(KeyData::LongData(*tick)),
    });
    kv_pairs.push(NameDataPair {
        name: "event_name".to_string(),
        data: Some(KeyData::StrData(event.name().to_string())),
    });
    kv_pairs.push(NameDataPair {
        name: "round".to_string(),
        data: Some(KeyData::LongData(round)),
    });
    

    kv_pairs
}

pub fn match_data_to_game_event(event_name: &str, wanted: &String) -> bool {
    return event_name.contains(wanted);
}

impl Demo {
    pub fn parse_game_events(&mut self, game_event: CSVCMsg_GameEvent) -> (Vec<GameEvent>, bool) {
        let mut game_events: Vec<GameEvent> = Vec::new();
        let mut connect_tick = false;
        match &self.event_map {
            Some(ev_desc_map) => {
                let event_desc = &ev_desc_map[&game_event.eventid()];

                if self.event_name.len() > 0 {
                    if match_data_to_game_event(event_desc.name(), &self.event_name) {
                        let name_data_pairs = gen_name_val_pairs(
                            &game_event,
                            &event_desc,
                            &self.tick,
                            &self.userid_sid_map,
                            &self.players,
                            self.round,
                            &self.entities,
                            &self.wanted_props

                        );

                        game_events.push({
                            GameEvent {
                                name: event_desc.name().to_owned(),
                                fields: name_data_pairs,
                            }
                        })
                    }
                } else {
                    {
                        let name_data_pairs = gen_name_val_pairs(
                            &game_event,
                            &event_desc,
                            &self.tick,
                            &self.userid_sid_map,
                            &self.players,
                            self.round,
                            &self.entities,
                            &self.wanted_props
                        );
                        game_events.push({
                            GameEvent {
                                name: event_desc.name().to_owned(),
                                fields: name_data_pairs,
                            }
                        })
                    }
                }
            }
            None => {
                panic!("Game event was not found in event list passed earlier");
            }
        }
        (game_events, connect_tick)
    }
    pub fn parse_game_event_map(&mut self, event_list: CSVCMsg_GameEventList) {
        let mut hm: HashMap<i32, Descriptor_t, RandomState> = HashMap::default();

        for event_desc in event_list.descriptors {
            hm.insert(event_desc.eventid(), event_desc);
        }
        self.event_map = Some(hm);
    }
}
