use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::demo_parsing::*;
use crate::parsing::parser_settings::*;
pub use crate::parsing::variants::*;
use ahash::HashMap;
#[cfg(feature = "blosc")]
use hdf5::filters::blosc_set_nthreads;
use hdf5::{File, H5Type, Result};
use itertools::Itertools;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use ndarray::arr1;
use ndarray::{arr2, s};
use polars::export::regex::internal::Inst;
use polars::series::Series;
use rayon::prelude::IntoParallelRefIterator;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use std::u8;

use super::cache::WriteCache;

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
        self.speed();
        //self.parse_bytes(vec![]);
        //self.indicies_modify();
    }
    pub fn speed(&mut self) {
        let mut rc = ReadCache::new(&self.bytes);
        rc.read_index();
        let mut wanted_bytes = vec![];
        let (dt_start, ge_start) = rc.read_dt_ge_map();
        wanted_bytes.push(dt_start);
        wanted_bytes.push(ge_start);

        let map = rc.get_eid_cls_map();
        self.state.eid_cls_history = map;

        let mut byt = rc.read_by_id(529);
        let before = Instant::now();

        let sts = rc.read_stringtables();
        println!("ST {:?}", sts);
        wanted_bytes.extend(byt);

        self.parse_bytes(wanted_bytes);
        println!("SPEED {:2?}", before.elapsed());
    }

    pub fn parse_bytes(&mut self, wanted_bytes: Vec<u64>) {
        let byte_readers = ByteReader::get_byte_readers(&self.bytes, wanted_bytes);
        let n_byte_readers = byte_readers.len();

        for mut byte_reader in byte_readers {
            while byte_reader.byte_idx < byte_reader.bytes.len() as usize {
                self.state.frame_started_at = byte_reader.byte_idx as u64;
                let (cmd, tick) = byte_reader.read_frame();
                self.state.tick = tick;
                self.parse_cmd(cmd, &mut byte_reader);
                if n_byte_readers > 1 {
                    break;
                }
            }
        }
    }
    pub fn indicies_modify(&mut self) -> Vec<u64> {
        /*
        Takes the vector with every updated packet ent index
        and transforms into pidx: Vec<(tick, entid)> pairs.
        */
        let mut wc = WriteCache::new(&self.bytes);

        //wc.write_maps(self.state.dt_started_at, self.state.ge_map_started_at);
        wc.write_packet_ents(&self.state.test, &self.maps.serverclass_map);
        let before = Instant::now();
        wc.write_eid_cls_map(&self.state.eid_cls_history);
        wc.write_dt_ge_map(self.state.dt_started_at, self.state.ge_map_started_at);
        wc.write_game_events(&self.state.game_event_history);
        wc.write_stringtables(&self.state.stringtable_history);
        wc.flush();

        let mut rc = ReadCache::new(&self.bytes);
        rc.read_index();
        let map = rc.get_eid_cls_map();
        self.state.eid_cls_history = map;

        let byt = rc.read_by_id(558);
        let before = Instant::now();
        println!("{:?}", byt);
        self.parse_bytes(byt);
        println!("SPEED {:2?}", before.elapsed());
        vec![]
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
        //println!("{} {}", msg, self.state.tick);
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
