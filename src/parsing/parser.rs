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
use hashbrown::HashMap;
use netmessages::CSVCMsg_PacketEntities;
use protobuf;
use protobuf::reflect::MessageDescriptor;
use protobuf::Message;
use pyo3::prelude::*;
use rayon;
use std::any::Any;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time;
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
    pub event_map: Option<HashMap<i32, Descriptor_t>>,
    pub dt_map: Arc<Mutex<Option<HashMap<String, CSVCMsg_SendTable>>>>,
    pub serverclass_map: Arc<Mutex<HashMap<u16, ServerClass>>>,
    pub entities: Arc<Mutex<HashMap<u32, Option<Entity>>>>,
    pub bad: Vec<String>,
    pub stringtables: Vec<StringTable>,
    pub players: Vec<UserInfo>,
    pub parse_props: bool,
    pub game_events: Vec<GameEvent>,
    pub event_name: String,
    pub cnt: i32,
    pub wanted_props: Vec<String>,
    pub handles: Vec<Option<JoinHandle<()>>>,
    pub threads_spawned: i32,
    pub closed_handles: i32,
    pub pool: rayon::ThreadPool,
    pub pool2: rayon::ThreadPool,
    pub last_pool: bool,
    pub pcnt: Arc<Mutex<i64>>,
}

impl Demo {
    pub fn parse_frame(&mut self, props_names: &Vec<String>) {
        // Main loop
        while self.fp < self.bytes.len() as usize {
            let f = self.read_frame();
            self.tick = f.tick;
            self.parse_cmd(f.cmd);
        }
    }

    pub fn read_frame(&mut self) -> Frame {
        let f = Frame {
            cmd: self.read_byte(),
            tick: self.read_i32(),
            playerslot: self.read_byte(),
        };
        //println!("{}", self.class_bits);
        //println!("TICK: {}, HANDLES: {}", f.tick, self.handles.len());
        f
    }

    pub fn join_handles(&mut self) {
        for handle in &mut self.handles {
            let h = handle.take();
            match h {
                Some(handle) => {
                    self.closed_handles += 1;
                    handle.join().unwrap();
                }
                None => {}
            }
        }
    }

    pub fn parse_cmd(&mut self, cmd: u8) {
        //println!("{}", cmd);
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
                // GAME EVENT
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
                // GAME EVENT LIST
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
                //PACKET ENTITIES
                26 => {
                    if parse_props {
                        let pack_ents: CSVCMsg_PacketEntities =
                            Message::parse_from_bytes(data).unwrap();

                        let now = Instant::now();

                        let cloned_ents = Arc::clone(&self.entities);
                        let cloned_dt_map = Arc::clone(&self.dt_map);
                        let serverclass_map = Arc::clone(&self.serverclass_map);
                        let clsbits = self.class_bits.clone();
                        let tick = self.tick.clone();
                        let elapsed = now.elapsed();

                        self.threads_spawned += 1;
                        //println!("BEFORE");
                        self.pool2.spawn(move || {
                            Demo::parse_packet_entities(
                                pack_ents,
                                parse_props.clone(),
                                clsbits,
                                cloned_ents.clone(),
                                cloned_dt_map.clone(),
                                tick.clone(),
                                serverclass_map.clone(),
                            )
                        });
                        //self.pool2.join(oper_a, oper_b)
                        //println!("AFTER");
                        //rayon::join(oper_a, oper_b)
                        //println!("{}", self.threads_spawned);
                    }
                }
                // CREATE STRING TABLE
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
                // UPDATE STRING TABLE
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
