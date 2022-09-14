pub mod data_table;
pub mod entities;
pub mod game_events;
pub mod header;
pub mod newbitreader;
pub mod read_bits;
pub mod read_bytes;
pub mod stringtables;

use crate::data_table::ServerClass;
use crate::entities::Entity;
use crate::entities::Prop;
use crate::game_events::HurtEvent;
use crate::header::Header;

use crate::netmessages::CSVCMsg_PacketEntities;
use crate::protobuf::Message;
use crate::stringtables::StringTable;
use csgoproto::netmessages;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::CSVCMsg_CreateStringTable;
use csgoproto::netmessages::CSVCMsg_GameEvent;
use csgoproto::netmessages::CSVCMsg_GameEventList;
use csgoproto::netmessages::CSVCMsg_SendTable;
use protobuf;
use protobuf::reflect::MessageDescriptor;
use std::any::Any;
use std::collections::HashMap;
use std::time::Instant;
use std::vec;
use stringtables::UserInfo;

#[allow(dead_code)]
struct Frame {
    cmd: u8,
    tick: i32,
    playerslot: u8,
}

struct Demo {
    fp: usize,
    tick: i32,
    cmd: u8,
    bytes: Vec<u8>,
    class_bits: u32,
    msg_map: Vec<MessageDescriptor>,
    event_list: Option<CSVCMsg_GameEventList>,
    event_vec: Option<Vec<Descriptor_t>>,
    dt_map: Option<HashMap<String, CSVCMsg_SendTable>>,
    serverclass_map: HashMap<u16, ServerClass>,
    entities: Option<HashMap<u32, Option<Entity>>>,
    bad: Vec<String>,
    stringtables: Vec<StringTable>,
    players: Vec<UserInfo>,
    data: Vec<f32>,
    cnt: i32,
}

impl Demo {
    fn parse_frame(&mut self) {
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
        f
    }

    fn parse_cmd(&mut self, cmd: u8) {
        match cmd {
            1 => self.parse_packet(),
            2 => self.parse_packet(),
            6 => self.parse_datatable(),
            //12 => self.parse_string_table(),
            _ => {} //panic!("UNK CMD"),
        }
    }

    fn parse_packet(&mut self) {
        self.fp += 160;
        let packet_len = self.read_i32();
        let goal_inx = self.fp + packet_len as usize;

        while self.fp < goal_inx {
            let msg = self.read_varint();
            let size = self.read_varint();
            let data = self.read_n_bytes(size);

            match msg as i32 {
                25 => {
                    let game_event: CSVCMsg_GameEvent = Message::parse_from_bytes(&data).unwrap();
                    self.parse_game_events(game_event);
                }
                30 => {
                    let event_list: CSVCMsg_GameEventList =
                        Message::parse_from_bytes(&data).unwrap();
                    self.parse_game_event_list(event_list)
                }
                26 => {
                    let pack_ents: CSVCMsg_PacketEntities =
                        Message::parse_from_bytes(data).unwrap();
                    self.parse_packet_entities(pack_ents);
                }
                12 => {
                    let string_table: CSVCMsg_CreateStringTable =
                        Message::parse_from_bytes(&data).unwrap();
                    self.create_string_table(string_table);
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let x = netmessages::file_descriptor();
    let y = x.messages();
    let mut v: Vec<MessageDescriptor> = Vec::new();

    let mut cnt = 0;
    for x in y {
        //println!("{cnt} {:?}", x.name());
        cnt += 1;
    }

    let now = Instant::now();
    let mut d = Demo {
        bytes: std::fs::read("/home/laiho/Documents/demos/rclonetest/q.dem").unwrap(),
        fp: 0,
        cmd: 0,
        tick: 0,
        msg_map: v,
        event_list: None,
        event_vec: None,
        dt_map: Some(HashMap::new()),
        class_bits: 0,
        serverclass_map: HashMap::new(),
        entities: Some(HashMap::new()),
        bad: Vec::new(),
        stringtables: Vec::new(),
        players: Vec::new(),
        data: Vec::new(),
        cnt: 0,
    };

    let h: Header = d.parse_header();
    d.parse_frame();

    println!("{:?}", d.data);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    for (k, v) in d.entities.unwrap().iter() {
        if v.is_some() {
            if v.as_ref().unwrap().props.len() > 500 {
                println!("{k} {:?}", &v.as_ref().unwrap().props);
            }
        }
    }
    for p in d.players {
        println!("{:?} {} {}", &p.name[..30], p.xuid, p.entity_id);
    }
}
