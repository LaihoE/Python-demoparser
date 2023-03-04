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
use itertools::Itertools;
use memmap2::Mmap;
use polars::prelude::NamedFrom;
use polars::series::Series;
use protobuf::Message;
use pyo3::prelude::*;
use std::collections::HashMap;

pub struct GameEventHistory {
    pub byte: i32,
    pub tick: i32,
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
            tick: self.state.tick as i32,
        });
        let event_desc = &self.maps.event_map.as_ref().unwrap()[&game_event.eventid()];
        if event_desc.name() == self.settings.event_name || self.settings.event_name == "" {
            let name_data_pairs =
                self.gen_name_val_pairs(&game_event, event_desc, &self.state.tick);

            let ge = GameEvent {
                byte: self.state.frame_started_at as usize,
                name: event_desc.name().to_string(),
                fields: name_data_pairs,
                tick: self.state.tick,
                id: game_event.eventid(),
            };
            self.state.game_events.push(ge);
        }
    }

    pub fn parse_game_event_map(&mut self, byte_reader: &mut ByteReader, size: usize) {
        self.state.ge_map_started_at = self.state.frame_started_at;
        let wanted_bytes = &self.bytes[byte_reader.byte_idx..byte_reader.byte_idx + size as usize];
        byte_reader.skip_n_bytes(size.try_into().unwrap());
        let game_event_list: CSVCMsg_GameEventList =
            Message::parse_from_bytes(wanted_bytes).unwrap();

        let mut hm: HashMap<i32, Descriptor_t, RandomState> = HashMap::default();
        for event_desc in game_event_list.descriptors {
            self.maps
                .event_name_to_id
                .insert(event_desc.name().to_string(), event_desc.eventid());
            hm.insert(event_desc.eventid(), event_desc);
        }
        self.maps.event_map = Some(hm);
    }
    pub fn filter_to_vec<Wanted>(v: impl IntoIterator<Item = impl TryInto<Wanted>>) -> Vec<Wanted> {
        v.into_iter().filter_map(|x| x.try_into().ok()).collect()
    }

    pub fn series_from_pairs(pairs: Vec<&NameDataPair>, name: &String) -> Series {
        let only_data: Vec<KeyData> = pairs.iter().map(|x| x.data.clone()).collect();
        let s = match pairs[0].data_type {
            1 => Series::new(name, &Parser::filter_to_vec::<String>(only_data)),
            2 => Series::new(name, &Parser::filter_to_vec::<f32>(only_data)),
            3 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            4 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            5 => Series::new(name, &Parser::filter_to_vec::<i64>(only_data)),
            6 => Series::new(name, &Parser::filter_to_vec::<bool>(only_data)),
            7 => Series::new(name, &Parser::filter_to_vec::<u64>(only_data)),
            _ => panic!("Keydata got unknown type: {}", pairs[0].data_type),
        };
        s
    }

    pub fn series_from_events(&self, events: &Vec<GameEvent>) -> Vec<Series> {
        // Example [Hashmap<"distance": 21.0>, Hashmap<"distance": 24.0>, Hashmap<"name": "Steve">]
        // ->
        // Hashmap<"distance": [21.0, 24.0], "name": ["Steve"]>,
        // -> Series::new("distance", [21.0, 24.0]) <-- needs to be mapped as "f32" not as enum(KeyData)
        let pairs: Vec<NameDataPair> = events.iter().map(|x| x.fields.clone()).flatten().collect();
        let per_key_name = pairs.iter().into_group_map_by(|x| &x.name);
        let mut series = vec![];
        for (name, vals) in per_key_name {
            let s = Parser::series_from_pairs(vals, name);
            series.push(s);
        }
        series.sort_by_key(|x| x.name().to_string());
        series
    }
    fn parse_key_steamid(&self, key: &Key_t) -> KeyData {
        let user_id = key.val_short();

        match self.maps.userid_sid_map.get(&(user_id as u32)) {
            None => KeyData::Uint64(0),
            Some(u) => {
                return KeyData::Uint64(*u);
            }
        }
    }
    fn get_tick_props(&self, pair: &NameDataPair, prefix: &str) -> Option<Vec<NameDataPair>> {
        // Find wanted entity, return none on all failed branches
        let wanted_entity = match pair.data {
            KeyData::Uint64(steamid) => {
                let entity = match self.maps.sid_entid_map.get(&steamid) {
                    Some(entid) => match self.state.entities.get(&(*entid as i32)) {
                        Some(ent) => Some(ent),
                        None => None,
                    },
                    None => None,
                };
                entity
            }
            _ => None,
        };
        let mut found_props = vec![];
        for prop_name in &self.settings.collect_props {
            let prop_idx = match self.maps.name_entid_prop.get(prop_name) {
                Some(idx) => *idx,
                None => {
                    found_props.push(NameDataPair {
                        name: prefix.to_string() + &prop_name,
                        data: KeyData::Float(0.0),
                        data_type: 2,
                    });
                    continue;
                }
            };
            match wanted_entity {
                Some(ent) => {
                    match ent.props.get(prop_idx) {
                        Some(prop) => {
                            let pair = match prop {
                                Some(p) => {
                                    let data = KeyData::from(p.clone());
                                    let data_type = keydata_type_from_enum(&data);
                                    NameDataPair {
                                        name: prefix.to_string() + &prop_name,
                                        data: data,
                                        data_type: data_type,
                                    }
                                }
                                None => NameDataPair {
                                    name: prefix.to_string() + &prop_name,
                                    data: KeyData::Float(0.0),
                                    data_type: 2,
                                },
                            };
                            found_props.push(pair)
                        }
                        None => {
                            found_props.push(NameDataPair {
                                name: prefix.to_string() + &prop_name,
                                data: KeyData::Float(0.0),
                                data_type: 2,
                            });
                        }
                    };
                }
                None => {
                    found_props.push(NameDataPair {
                        name: prefix.to_string() + &prop_name,
                        data: KeyData::Float(0.0),
                        data_type: 2,
                    });
                }
            }
        }
        return Some(found_props);
    }

    pub fn gen_name_val_pairs(
        &self,
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
            let new_val = match desc.name() {
                // Replace userids with steamids
                "userid" => self.parse_key_steamid(ge),
                "attacker" => self.parse_key_steamid(ge),
                "assister" => self.parse_key_steamid(ge),
                _ => val,
            };
            let data_type = match desc.name() {
                "userid" => 7,
                "attacker" => 7,
                "assister" => 7,
                _ => ge.type_(),
            };
            kv_pairs.push(NameDataPair {
                name: desc.name().to_owned(),
                data: new_val,
                data_type: data_type,
            })
        }
        let mut new_pairs = vec![];
        for pair in &kv_pairs {
            let pairs = match pair.name.as_str() {
                "userid" => self.get_tick_props(&pair, "user_"),
                "attacker" => self.get_tick_props(&pair, "attacker_"),
                "assister" => self.get_tick_props(&pair, "assister_"),
                _ => None,
            };
            match pairs {
                Some(pairs) => new_pairs.extend(pairs),
                None => {}
            }
        }
        kv_pairs.extend(new_pairs);
        kv_pairs.push(NameDataPair {
            name: "tick".to_owned(),
            data: KeyData::Long(*tick),
            data_type: 3,
        });
        kv_pairs
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
fn keydata_type_from_enum(value: &KeyData) -> i32 {
    match value {
        KeyData::Str(_) => 1,
        KeyData::Float(_) => 2,
        KeyData::Long(_) => 3,
        KeyData::Short(_) => 4,
        KeyData::Byte(_) => 5,
        KeyData::Bool(_) => 6,
        KeyData::Uint64(_) => 7,
    }
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
    /*
    pub fn from_pdata(pdata: &PropData) -> Self {
        match pdata {
            PropData::F32(f) => KeyData::Float(*f),
            PropData::I32(f) => KeyData::Long(*f),
            PropData::String(f) => KeyData::Str(f.to_string()),
            _ => panic!("not yet suppored"),
        }
    }
    */
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
