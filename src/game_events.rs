use std::any::Any;

use crate::Demo;
use csgoproto::netmessages::csvcmsg_game_event::Key_t;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_GameEvent;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use protobuf::Message;

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
#[derive(Debug)]
pub struct NameDataPair {
    pub name: String,
    pub data: KeyData,
}
#[derive(Debug)]
pub struct GameEvent {
    name: String,
    fields: Vec<NameDataPair>,
}

pub fn parse_game_event(game_event: &CSVCMsg_GameEvent, event: &Descriptor_t) -> HurtEvent {
    let mut he = HurtEvent::default();
    let mut cnt = 0;
    for key in &game_event.keys {
        match event.keys[cnt].name() {
            "userid" => he.userid = key.val_short(),
            "attacker" => he.attacker = key.val_short(),
            "health" => he.health = key.val_byte(),
            "armor" => he.armor = key.val_byte(),
            "weapon" => he.weapon = key.val_string().try_into().unwrap(),
            "dmg_health" => he.dmg_health = key.val_short(),
            "dmg_armor" => he.dmg_armor = key.val_byte(),
            "hitgroup" => he.hitgroup = key.val_byte(),
            _ => println!("POOP"),
        }
        cnt += 1;
    }
    he
}
pub fn gen_name_val_pairs(
    game_event: &CSVCMsg_GameEvent,
    event: &Descriptor_t,
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
    kv_pairs
}

pub fn _match_data_to_game_event(namedatavec: Vec<NameDataPair>, event_name: &str) {
    //println!("{:#?} {:#?}", event_name, namedatavec);
    if !event_name.contains("player")
        && !event_name.contains("weapon")
        && !event_name.contains("fire")
        && !event_name.contains("item")
    {
        println!("{:#?}", event_name);
    }
}

impl Demo {
    pub fn parse_game_events(&self, game_event: CSVCMsg_GameEvent) -> Vec<GameEvent> {
        let mut game_events: Vec<GameEvent> = Vec::new();
        for event_desc in self.event_vec.as_ref().unwrap() {
            if event_desc.eventid() == game_event.eventid() {
                let name_data_pairs = gen_name_val_pairs(&game_event, event_desc);
                game_events.push({
                    GameEvent {
                        name: event_desc.name().to_owned(),
                        fields: name_data_pairs,
                    }
                })
            }
        }
        game_events
    }

    pub fn parse_game_event_list(&mut self, event_list: CSVCMsg_GameEventList) {
        self.event_vec = Some(event_list.descriptors);
    }
}
