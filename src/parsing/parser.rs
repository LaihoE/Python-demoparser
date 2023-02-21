use crate::parsing::demo_parsing::*;
use crate::parsing::parser_settings::*;
pub use crate::parsing::variants::*;
use ahash::HashMap;
use itertools::Itertools;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use polars::export::regex::internal::Inst;
use polars::series::Series;
use rayon::prelude::IntoParallelRefIterator;
use std::sync::Arc;
use std::time::Instant;
use std::u8;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub struct Parser {
    pub maps: Maps,
    pub settings: ParserSettings,
    pub state: ParserState,
    pub bytes: Arc<Mmap>,
    // General purpose int vec, for perf reasons
}
/*
FRAME -> CMD -> NETMESSAGE----------> TYPE --> Packet entities
             -> DATATABLE                  --> Game events
             -> ...(not used)              --> Stringtables
*/

impl Parser {
    pub fn start_parsing(&mut self) {
        self.parse_bytes(vec![]);
    }

    pub fn parse_bytes(&mut self, wanted_bytes: Vec<u64>) {
        for _ in 0..5000 {
            self.state.entities.push((
                1111111,
                Entity {
                    class_id: 0,
                    entity_id: 1111111,
                    //props: HashMap::default(),
                },
            ));
        }
        let byte_readers = ByteReader::get_byte_readers(&self.bytes, vec![]);

        for mut byte_reader in byte_readers {
            while byte_reader.byte_idx < byte_reader.bytes.len() as usize {
                let (cmd, tick) = byte_reader.read_frame();
                self.state.tick = tick;
                self.parse_cmd(cmd, &mut byte_reader);
            }
        }
        self.indicies_modify()
    }
    pub fn indicies_modify(&mut self) {
        /*
        Takes the vector with every updated packet ent index
        and transforms into pidx: Vec<(tick, entid)> pairs.
        */
        use rayon::iter::ParallelIterator;
        let before = Instant::now();
        let mut cur_tick = 0;
        let mut cur_ent = 0;
        let mut my_p = HashMap::default();

        // 1d vec --> {pidx: Vec<(tick, entid)>}

        for i in 0..self.state.workhorse_idx {
            let val = self.state.workhorse[i];
            match val {
                999999999 => {
                    cur_tick = self.state.workhorse[i + 1];
                }
                111111111 => {
                    cur_ent = self.state.workhorse[i + 1];
                }
                _ => {
                    my_p.entry(val).or_insert(vec![]).push((cur_tick, cur_ent));
                }
            }
        }

        for (k, v) in &my_p {
            for i in v {
                println!("{:?}", i);
            }
        }

        println!("{:2?}", before.elapsed());
    }
    pub fn parse_cmd(&mut self, cmd: u8, byte_reader: &mut ByteReader) {
        match cmd {
            1 => self.messages_from_packet(byte_reader),
            2 => self.messages_from_packet(byte_reader),
            6 => self.parse_datatable(byte_reader),
            7 => { // signals end of demo
            }
            _ => {}
        }
    }
    pub fn msg_handler(&mut self, msg: u32, size: u32, byte_reader: &mut ByteReader) {
        match msg {
            12 => self.create_string_table(byte_reader, size as usize),
            13 => self.update_string_table_msg(byte_reader, size as usize),
            25 => self.parse_game_event(byte_reader, size as usize),
            26 => self.parse_packet_entities(byte_reader, size as usize),
            30 => self.parse_game_event_map(byte_reader, size as usize),
            _ => {
                byte_reader.skip_n_bytes(size);
            }
        }
    }
    pub fn messages_from_packet(&mut self, byte_reader: &mut ByteReader) {
        byte_reader.skip_n_bytes(160);
        let packet_len = byte_reader.read_i32();
        let goal_inx = byte_reader.byte_idx + packet_len as usize;
        while byte_reader.byte_idx < goal_inx {
            let msg = byte_reader.read_varint();
            let size = byte_reader.read_varint();
            self.msg_handler(msg, size, byte_reader);
        }
    }
}
