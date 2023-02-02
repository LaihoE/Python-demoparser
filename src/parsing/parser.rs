use super::game_events::GameEvent;
use super::utils::TYPEHM;
use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::cache::cache_reader::*;
use crate::parsing::cache::cache_writer::WriteCache;

use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::parse_packet_entities;
use crate::parsing::entities::PacketEntsOutput;
use crate::parsing::parser_settings::*;
use crate::parsing::players::Players;
use crate::parsing::read_bytes::ByteReader;
use crate::parsing::stringtables::UserInfo;
pub use crate::parsing::variants::*;
use ahash::HashSet;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use itertools::Itertools;
use memmap2::Mmap;
use polars::export::num::NumCast;
use polars::prelude::NamedFrom;
use polars::series::Series;
use rayon::prelude::IntoParallelRefIterator;
use sha256;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::u8;

pub struct Parser {
    pub maps: Maps,
    pub settings: ParserSettings,
    pub state: ParserState,
    pub bytes: Arc<Mmap>,
    // General purpose int vec, for perf reasons
    pub tasks: Vec<MsgBluePrint>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MsgBluePrint {
    pub msg: u32,
    pub start_idx: usize,
    pub end_idx: usize,
    pub tick: i32,
    pub byte: usize,
}
#[derive(Debug)]
pub enum JobResult {
    PacketEntities(PacketEntsOutput),
    GameEvents(Vec<GameEvent>),
    StringTables(Vec<UserInfo>),
    None,
}

/*
FRAME -> CMD -> NETMESSAGE----------> TYPE --> Packet entities
             -> DATATABLE                  --> Game events
             -> ...(not used)              --> Stringtables
*/

impl Parser {
    fn get_byte_readers(&self, start_pos: Vec<u64>) -> Vec<ByteReader> {
        if start_pos.len() == 0 {
            return vec![ByteReader::new(self.bytes.clone(), false, 1072)];
        }
        let mut readers = vec![];
        for pos in start_pos {
            readers.push(ByteReader::new(self.bytes.clone(), true, pos as usize));
        }
        return readers;
    }

    pub fn start_parsing(&mut self) -> Vec<Series> {
        match ReadCache::get_cache_if_exists(&self.bytes) {
            Some(mut cache) => {
                // println!("Using cache");
                // Bytes where our wanted ticks start
                let wanted_bytes = cache.get_player_messages();
                self.parse_bytes(wanted_bytes);

                let jobresults = self.compute_jobs(&mut cache);
                jobresults
            }
            // NO CACHE FOUND
            None => {
                self.parse_bytes(vec![]);
                let jobresults = self.compute_jobs_no_cache();
                let cache_path = ReadCache::get_cache_path(&self.bytes);

                let mut wc = WriteCache::new(
                    &cache_path,
                    jobresults,
                    self.state.dt_started_at,
                    self.state.ge_map_started_at,
                );
                wc.write_all_caches(&self.maps.serverclass_map);
                drop(wc);
                match ReadCache::get_cache_if_exists(&self.bytes) {
                    Some(mut cache) => self.compute_jobs(&mut cache),
                    None => panic!("FAILED TO READ WRITTEN CACHE"),
                }
            }
        }
    }
    pub fn parse_bytes(&mut self, wanted_bytes: Vec<u64>) -> Vec<MsgBluePrint> {
        let v = vec![];
        let byte_readers = self.get_byte_readers(wanted_bytes);

        for mut byte_reader in byte_readers {
            let mut frames_parsed = 0;
            while byte_reader.byte_idx < byte_reader.bytes.len() as usize {
                if byte_reader.single && frames_parsed > 0 {
                    break;
                }
                let (cmd, tick) = byte_reader.read_frame();
                self.state.tick = tick;
                self.parse_cmd(cmd, &mut byte_reader);
                frames_parsed += 1;
            }
        }
        v
    }

    pub fn parse_blueprints(&mut self) -> Vec<JobResult> {
        // let unqt: Vec<&MsgBluePrint> = combined_tasks.iter().unique_by(|x| x.tick).collect();
        // println!("UNIQ OF THEM: {}", unq.len());

        let mut opt = None;
        for t in &self.tasks {
            if t.msg == 30 {
                opt = Some(t.clone());
                break;
            }
        }
        if opt.is_some() {
            self.parse_game_event_map(&opt.unwrap());
        }
        use rayon::iter::ParallelIterator;

        self.tasks
            .iter()
            .map(|t| {
                Parser::msg_handler(
                    &t,
                    &self.bytes,
                    &self.maps.serverclass_map,
                    &self.maps.event_map.as_ref().unwrap(),
                )
            })
            .collect()
    }

    #[inline(always)]
    pub fn parse_cmd(&mut self, cmd: u8, byte_reader: &mut ByteReader) {
        match cmd {
            1 => self.parse_packet(byte_reader),
            2 => self.parse_packet(byte_reader),
            6 => {
                self.state.dt_started_at = (byte_reader.byte_idx - 6) as u64;
                self.parse_datatable(byte_reader)
            }
            _ => {}
        }
    }

    #[inline(always)]
    pub fn parse_packet(&mut self, byte_reader: &mut ByteReader) {
        let packet_started_at = byte_reader.byte_idx - 6;
        byte_reader.byte_idx += 160;
        let packet_len = byte_reader.read_i32();
        let goal_inx = byte_reader.byte_idx + packet_len as usize;
        while byte_reader.byte_idx < goal_inx {
            let msg = byte_reader.read_varint();
            let size = byte_reader.read_varint();
            // let (msg, size) = byte_reader.read_two_varints();
            // Get byte boundaries for this msg
            let before_inx = byte_reader.byte_idx.clone();
            byte_reader.byte_idx += size as usize;
            let after_inx = byte_reader.byte_idx.clone();
            // Information needed to parse a msg, passed to threads as a "job"
            let msg_blueprint = MsgBluePrint {
                msg: msg,
                start_idx: before_inx,
                end_idx: after_inx,
                tick: self.state.tick,
                byte: packet_started_at,
            };
            if msg == 25 || msg == 26 || msg == 12 || msg == 13 || msg == 30 {
                self.tasks.push(msg_blueprint);
            }
        }
    }

    #[inline(always)]
    pub fn msg_handler(
        blueprint: &MsgBluePrint,
        bytes: &Mmap,
        serverclass_map: &HashMap<u16, ServerClass, RandomState>,
        game_events_map: &HashMap<i32, Descriptor_t, RandomState>,
    ) -> JobResult {
        let wanted_event = "player_death";

        match blueprint.msg {
            26 => parse_packet_entities(blueprint, bytes, serverclass_map),
            25 => Parser::parse_game_events(blueprint, bytes, game_events_map, wanted_event),
            12 => Parser::create_string_table(blueprint, bytes),
            13 => Parser::update_string_table_msg(blueprint, bytes),
            _ => JobResult::None,
        }
    }
}
