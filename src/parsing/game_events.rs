use super::stringtables::UserInfo;
use crate::parsing::entities::Entity;
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

fn parse_key_steamid(key: &Key_t, uid_sid_map: &HashMap<u32, u64, RandomState>) -> KeyData {
    let user_id = key.val_short();

    match uid_sid_map.get(&(user_id as u32)) {
        None => KeyData::Long(0),
        Some(u) => {
            return KeyData::Uint64(*u);
        }
    }
}

pub fn parse_props(
    key: &Key_t,
    entities: &[(u32, Entity)],
    wanted_props: &Vec<String>,
    og_names: &[String],
    prefix: String,
    uid_eid_map: &HashMap<u32, u64, RandomState>,
) -> Vec<NameDataPair> {
    let user_id = key.val_short();
    let ent_id = uid_eid_map.get(&(user_id as u32)).unwrap_or(&999999);
    let mut all_pairs = vec![];
    match entities.get(*ent_id as usize) {
        None => all_pairs,
        Some(ent) => {
            for wanted_prop_inx in 0..og_names.len() {
                match ent.1.props.get(&wanted_props[wanted_prop_inx]) {
                    Some(p) => {
                        match &p.data {
                            PropData::F32(f) => {
                                all_pairs.push(NameDataPair {
                                    name: prefix.clone() + &og_names[wanted_prop_inx].to_string(),
                                    data: Some(KeyData::Float(*f)),
                                });
                            }
                            PropData::I32(i) => {
                                all_pairs.push(NameDataPair {
                                    name: prefix.clone() + &og_names[wanted_prop_inx].to_string(),
                                    data: Some(KeyData::Long(*i)),
                                });
                            }
                            PropData::String(s) => {
                                all_pairs.push(NameDataPair {
                                    name: prefix.clone() + &og_names[wanted_prop_inx].to_owned(),
                                    data: Some(KeyData::Str(s.clone())),
                                });
                            }
                            PropData::VecXY(_) => {
                                // Handled by above sub-types and lets not create it here
                            }
                            _ => {
                                all_pairs.push(NameDataPair {
                                    name: prefix.clone() + &p.prop_name,
                                    data: None,
                                });
                            }
                        }
                    }
                    None => {
                        all_pairs.push(NameDataPair {
                            name: prefix.clone() + &og_names[wanted_prop_inx],
                            data: None,
                        });
                    }
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
        None => return KeyData::Str("None".to_string()),
        Some(sid) => {
            for player in players.values() {
                if &player.xuid == sid {
                    if key.type_() == 4 {
                        return KeyData::Str(
                            player
                                .name
                                .to_string()
                                .trim_matches(char::from(0))
                                .to_string(),
                        );
                    }
                }
            }
        }
    }
    KeyData::Str("None".to_string())
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
/*
1 => Some(KeyData::Str(key.val_string().to_owned())),
2 => Some(KeyData::Float(key.val_float())),
3 => Some(KeyData::Long(key.val_long())),
4 => Some(KeyData::Short(key.val_short().try_into().unwrap())),
5 => Some(KeyData::Byte(key.val_byte().try_into().unwrap())),
6 => Some(KeyData::Bool(key.val_bool())),
7 => Some(KeyData::Uint64(key.val_uint64())),
*/

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
    uid_sid_map: &HashMap<u32, u64, RandomState>,
    players: &HashMap<u64, UserInfo, RandomState>,
    round: i32,
    entities: &[(u32, Entity)],
    wanted_props: &Vec<String>,
    og_names: &[String],
    uid_eid_map: &HashMap<u32, u64, RandomState>,
) -> Vec<NameDataPair> {
    // Takes the msg and its descriptor and parses (name, val) pairs from it
    let mut kv_pairs: Vec<NameDataPair> = Vec::new();

    for i in 0..game_event.keys.len() {
        let ge = &game_event.keys[i];
        let desc = &event.keys[i];

        match desc.name() {
            "userid" => {
                let steamid = parse_key_steamid(ge, uid_sid_map);
                kv_pairs.push(NameDataPair {
                    name: "player_steamid".to_string(),
                    data: Some(steamid),
                });
                let steam_name = parse_key_steam_name(ge, players, uid_sid_map);
                kv_pairs.push(NameDataPair {
                    name: "player_name".to_string(),
                    data: Some(steam_name),
                });
                let props = parse_props(
                    ge,
                    entities,
                    wanted_props,
                    og_names,
                    "player_".to_string(),
                    uid_eid_map,
                );
                for p in props {
                    kv_pairs.push(p);
                }
            }
            "attacker" => {
                let steamid = parse_key_steamid(ge, uid_sid_map);
                kv_pairs.push(NameDataPair {
                    name: "attacker_steamid".to_string(),
                    data: Some(steamid),
                });
                let steam_name = parse_key_steam_name(ge, players, uid_sid_map);
                kv_pairs.push(NameDataPair {
                    name: "attacker_name".to_string(),
                    data: Some(steam_name),
                });
                let props = parse_props(
                    ge,
                    entities,
                    wanted_props,
                    og_names,
                    "attacker_".to_string(),
                    uid_eid_map,
                );
                for p in props {
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
        data: Some(KeyData::Long(*tick)),
    });
    kv_pairs.push(NameDataPair {
        name: "event_name".to_string(),
        data: Some(KeyData::Str(event.name().to_string())),
    });
    kv_pairs.push(NameDataPair {
        name: "round".to_string(),
        data: Some(KeyData::Long(round)),
    });

    kv_pairs
}

pub fn match_data_to_game_event(event_name: &str, wanted: &String) -> bool {
    event_name.contains(wanted)
}

impl Demo {
    pub fn parse_game_events(&mut self, game_event: CSVCMsg_GameEvent) -> (Vec<GameEvent>, bool) {
        let mut game_events: Vec<GameEvent> = Vec::new();
        let connect_tick = false;
        match &self.event_map {
            Some(ev_desc_map) => {
                let event_desc = &ev_desc_map[&game_event.eventid()];

                if !self.event_name.is_empty() {
                    if match_data_to_game_event(event_desc.name(), &self.event_name) {
                        let name_data_pairs = gen_name_val_pairs(
                            &game_event,
                            event_desc,
                            &self.tick,
                            &self.userid_sid_map,
                            &self.players,
                            self.round,
                            &self.entities,
                            &self.wanted_props,
                            &self.friendly_p_names,
                            &self.uid_eid_map,
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
                            event_desc,
                            &self.tick,
                            &self.userid_sid_map,
                            &self.players,
                            self.round,
                            &self.entities,
                            &self.wanted_props,
                            &self.friendly_p_names,
                            &self.uid_eid_map,
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
