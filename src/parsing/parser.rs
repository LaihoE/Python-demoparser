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
use netmessages::CSVCMsg_PacketEntities;
use protobuf;
use protobuf::reflect::MessageDescriptor;
use protobuf::Message;
use pyo3::prelude::*;
use std::any::Any;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Instant;
use std::vec;

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
    pub event_vec: Option<Vec<Descriptor_t>>,
    pub dt_map: Option<HashMap<String, CSVCMsg_SendTable>>,
    pub serverclass_map: HashMap<u16, ServerClass>,
    pub entities: Option<HashMap<u32, Option<Entity>>>,
    pub bad: Vec<String>,
    pub stringtables: Vec<StringTable>,
    pub players: Vec<UserInfo>,
    pub parse_props: bool,
    pub game_events: Vec<GameEvent>,
    pub event_name: String,
}

impl Demo {
    pub fn parse_frame(&mut self, props_names: &Vec<String>) -> HashMap<String, Vec<f32>> {
        // Main loop
        let mut ticks_props: HashMap<String, Vec<f32>> = HashMap::new();
        for name in props_names {
            ticks_props.insert(name.to_string(), Vec::new());
        }
        let mut cc = 0;
        while self.fp < self.bytes.len() as usize {
            cc += 1;
            let f = self.read_frame();
            self.tick = f.tick;
            let props_this_tick: Vec<(String, f32)> =
                extract_props(&self.entities, props_names, &self.tick);
            for (k, v) in props_this_tick {
                ticks_props.entry(k).or_insert_with(Vec::new).push(v);
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
            _ => {} //panic!("UNK CMD"),
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
                    let game_event: CSVCMsg_GameEvent = Message::parse_from_bytes(&data).unwrap();
                    let game_events = self.parse_game_events(game_event);
                    self.game_events.extend(game_events);
                }
                30 => {
                    let event_list: CSVCMsg_GameEventList =
                        Message::parse_from_bytes(&data).unwrap();
                    self.parse_game_event_list(event_list)
                }
                26 => {
                    if parse_props {
                        let pack_ents: CSVCMsg_PacketEntities =
                            Message::parse_from_bytes(data).unwrap();
                        self.parse_packet_entities(pack_ents, parse_props);
                    }
                }
                12 => {
                    if parse_props {
                        let string_table: CSVCMsg_CreateStringTable =
                            Message::parse_from_bytes(&data).unwrap();
                        self.create_string_table(string_table);
                    }
                }
                _ => {}
            }
        }
    }
}
