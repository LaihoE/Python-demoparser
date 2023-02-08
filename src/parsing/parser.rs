use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::cache::cache_writer::WriteCache;
use crate::parsing::demo_parsing::*;
use crate::parsing::parser_settings::*;
pub use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use itertools::Itertools;
use memmap2::Mmap;
use polars::series::Series;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use smallvec::SmallVec;
use std::collections::HashMap;
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
    PacketEntitiesIndicies(Vec<EntityIndicies>),
    GameEvents(Vec<GameEvent>),
    StringTables(Vec<UserInfo>),
    None,
}

pub struct ParsingOutPut {
    pub df: Vec<Series>,
    pub events: Vec<Series>,
}

/*
FRAME -> CMD -> NETMESSAGE----------> TYPE --> Packet entities
             -> DATATABLE                  --> Game events
             -> ...(not used)              --> Stringtables
*/

impl Parser {
    pub fn start_parsing(&mut self) -> ParsingOutPut {
        match ReadCache::get_cache_if_exists(&self.bytes) {
            // CACHE FOUND
            Some(mut cache) => {
                let wanted_bytes = cache.get_player_messages();
                self.parse_bytes(wanted_bytes);
                let jobresults = self.compute_jobs_with_cache(&mut cache);
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
                    Some(mut cache) => self.compute_jobs_with_cache(&mut cache),
                    None => panic!("FAILED TO READ WRITTEN CACHE"),
                }
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
            12 => Parser::create_string_table(blueprint, bytes),
            13 => Parser::update_string_table_msg(blueprint, bytes),
            //25 => Parser::parse_game_events(blueprint, bytes, game_events_map, wanted_event),
            26 => Parser::parse_packet_entities(blueprint, bytes, serverclass_map),
            //26 => Parser::parse_packet_entities_indicies(blueprint, bytes, serverclass_map),
            _ => JobResult::None,
        }
    }
    #[inline(always)]
    pub fn parse_bytes(&mut self, wanted_bytes: Vec<u64>) {
        let uniq_bytes: Vec<&u64> = wanted_bytes.iter().dedup().collect();
        let byte_readers = ByteReader::get_byte_readers(&self.bytes, uniq_bytes);

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
    }
    #[inline(always)]
    pub fn parse_cmd(&mut self, cmd: u8, byte_reader: &mut ByteReader) {
        match cmd {
            1 => self.messages_from_packet(byte_reader),
            2 => self.messages_from_packet(byte_reader),
            6 => self.parse_datatable(byte_reader),
            _ => {}
        }
    }
    #[inline(always)]
    pub fn messages_from_packet(&mut self, byte_reader: &mut ByteReader) {
        let packet_started_at = byte_reader.byte_idx - 6;
        byte_reader.skip_n_bytes(160);
        let packet_len = byte_reader.read_i32();
        let goal_inx = byte_reader.byte_idx + packet_len as usize;
        while byte_reader.byte_idx < goal_inx {
            let msg = byte_reader.read_varint();
            let size = byte_reader.read_varint();
            // Get byte boundaries for this msg
            let before_inx = byte_reader.byte_idx.clone();
            byte_reader.skip_n_bytes(size);
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
    pub fn parse_blueprints(&mut self) -> Vec<JobResult> {
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
}
