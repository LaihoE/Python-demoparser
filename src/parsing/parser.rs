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
                self.state.frame_started_at = byte_reader.byte_idx as u64;
                let (cmd, tick) = byte_reader.read_frame();
                self.state.tick = tick;
                self.parse_cmd(cmd, &mut byte_reader);
            }
        }
        self.indicies_modify();
    }
    pub fn indicies_modify(&mut self) -> Vec<u64> {
        /*
        Takes the vector with every updated packet ent index
        and transforms into pidx: Vec<(tick, entid)> pairs.
        */
        let mut wc = WriteCache::new(&self.bytes);
        wc.write_maps(self.state.dt_started_at, self.state.ge_map_started_at);
        wc.write_packet_ents(&self.state.test, &self.maps.serverclass_map);
        let before = Instant::now();
        wc.flush();

        let mut rc = ReadCache::new(&self.bytes);
        rc.read_index();
        rc.read_by_id(189);
        println!("{:2?}", before.elapsed());
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
            //12 => self.create_string_table(byte_reader, size as usize),
            //13 => self.update_string_table_msg(byte_reader, size as usize),
            //25 => self.parse_game_event(byte_reader, size as usize),
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
