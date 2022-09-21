use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::Entity;
use crate::parsing::entities::Prop;
use crate::parsing::game_events::HurtEvent;
use crate::parsing::header::Header;

use crate::parsing::extract_props::extract_props;
use crate::parsing::read_bits::PropAtom;
use crate::parsing::read_bits::PropData;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
use csgoproto::netmessages;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_CreateStringTable;
use csgoproto::netmessages::CSVCMsg_GameEvent;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use csgoproto::netmessages::CSVCMsg_SendTable;
use fxhash::FxHashMap;
use netmessages::CSVCMsg_PacketEntities;
use protobuf;
use protobuf::reflect::MessageDescriptor;
use protobuf::Message;
use pyo3::prelude::*;
use std::any::Any;
use std::convert::TryInto;
use std::thread;
use std::time::Instant;
use std::vec;

use hashbrown::HashMap;

use numpy::ndarray::{Array1, ArrayD, ArrayView1, ArrayViewD, ArrayViewMutD, Zip};
use numpy::{
    datetime::{units, Timedelta},
    Complex64, IntoPyArray, PyArray1, PyArrayDyn, PyReadonlyArray1, PyReadonlyArrayDyn,
    PyReadwriteArray1, PyReadwriteArrayDyn,
};

use super::game_events::GameEvent;

#[allow(dead_code)]
pub struct Frame {
    pub cmd: u8,
    pub tick: i32,
    pub playerslot: u8,
}

pub struct Demo {
    pub fp: usize,
    pub tick: i32,
    pub cmd: u8,
    pub bytes: Vec<u8>,
    pub class_bits: u32,
    pub event_list: Option<CSVCMsg_GameEventList>,
    pub event_map: Option<HashMap<i32, Descriptor_t>>,
    pub serverclass_map: HashMap<u16, ServerClass>,
    pub entities: Option<HashMap<u32, Option<Entity>>>,
    pub bad: Vec<String>,
    pub stringtables: Vec<StringTable>,
    pub players: HashMap<u64, UserInfo>,
    pub parse_props: bool,
    pub game_events: Vec<GameEvent>,
    pub event_name: String,
    pub cnt: i32,
    pub wanted_props: Vec<String>,
}

impl Demo {
    pub fn parse_frame(&mut self, props_names: &Vec<String>) -> FxHashMap<String, Vec<f32>> {
        // Main loop
        let mut ticks_props: FxHashMap<String, Vec<f32>> = FxHashMap::default();
        for name in props_names {
            ticks_props.insert(name.to_string(), Vec::new());
        }

        while self.fp < self.bytes.len() as usize {
            let f = self.read_frame();
            self.tick = f.tick;
            //println!("{}", self.tick);
<<<<<<< HEAD

            let props_this_tick: Vec<(String, f32)> =
                extract_props(&self.entities, props_names, &self.tick, &self.players);
            for (k, v) in props_this_tick {
                ticks_props.entry(k).or_insert_with(Vec::new).push(v);
=======
            for player in &self.players {
                let props_this_tick: Vec<(String, f32)> =
                    extract_props(&self.entities, props_names, &self.tick, player.entity_id);
                for (k, v) in props_this_tick {
                    ticks_props.entry(k).or_insert_with(Vec::new).push(v);
                }
>>>>>>> no_lifetimes_stringtable
            }

            self.parse_cmd(f.cmd);
        }
        ticks_props
    }

    pub fn read_frame(&mut self) -> Frame {
        let f = Frame {
            cmd: self.read_byte(),
            tick: self.read_i32(),
            playerslot: self.read_byte(),
        };
        f
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
                25 => {
                    let game_event = Message::parse_from_bytes(&data);
                    match game_event {
                        Ok(ge) => {
                            let game_event = ge;
                            let game_events = self.parse_game_events(game_event, &self.players);
                            self.game_events.extend(game_events);
                        }
                        Err(e) => panic!(
                            "Failed to parse game event at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                30 => {
                    let event_list = Message::parse_from_bytes(&data);
                    match event_list {
                        Ok(ev) => {
                            let event_list = ev;
                            self.parse_game_event_list(event_list)
                        }
                        Err(e) => panic!(
                            "Failed to parse game event LIST at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                26 => {
                    if parse_props {
                        let pack_ents = Message::parse_from_bytes(data);
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
