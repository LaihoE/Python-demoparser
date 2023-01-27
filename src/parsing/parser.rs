use super::cache::ReadCache;
use super::cache::WriteCache;
use super::entities::SingleEntOutput;
use super::game_events::GameEvent;
use crate::parsing::cache;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::parse_packet_entities;
use crate::parsing::entities::PacketEntsOutput;
use crate::parsing::parser_settings::*;
use crate::parsing::players::Players;
use crate::parsing::read_bytes::ByteReader;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
pub use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use itertools::Itertools;
use memmap2::Mmap;
use polars::export::arrow::compute::filter;
use polars::export::regex::internal::Inst;
use polars::prelude::NamedFrom;
use polars::series::Series;
use rayon::prelude::*;
use sha256;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fs::metadata;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use std::u8;

pub struct Parser {
    pub maps: Maps,
    pub settings: ParserSettings,
    pub state: ParserState,
    pub bytes: Arc<Mmap>,
    // General purpose int vec, for perf reasons
    pub tasks: Vec<MsgBluePrint>,
}
#[derive(Debug, Clone)]
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
impl JobResult {
    pub fn is_stringtable(&self) -> bool {
        match self {
            JobResult::StringTables(_) => true,
            _ => false,
        }
    }
    pub fn is_game_event(&self) -> bool {
        match self {
            JobResult::GameEvents(_) => true,
            _ => false,
        }
    }
    pub fn is_packet_ent(&self) -> bool {
        match self {
            JobResult::PacketEntities(_) => true,
            _ => false,
        }
    }
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
    pub fn get_cache_path(&self) -> String {
        let file_hash = sha256::digest(&self.bytes[..10000]);
        let path = "/home/laiho/Documents/cache/".to_owned();
        path + &file_hash + &".zip"
    }

    pub fn get_cache_if_exists(&self) -> Option<ReadCache> {
        let cache_path = self.get_cache_path();
        println!("{}", cache_path);
        // If file exists
        match Path::new(&cache_path).exists() {
            true => Some(ReadCache::new(&cache_path)),
            false => None,
        }
    }
    pub fn get_wanted_bytes(&self, cache: &mut ReadCache) -> Vec<u64> {
        cache.read_stringtables();
        cache.read_game_events();

        let (ge_start, dt_start) = cache.read_maps();
        let mut wanted_bytes = cache.get_event_bytes_by_id(24);

        wanted_bytes.push(ge_start as u64);
        wanted_bytes.push(dt_start as u64);

        wanted_bytes.extend(cache.get_stringtables());
        wanted_bytes.extend(cache.find_delta_ticks(68, 2));

        wanted_bytes
    }

    pub fn start_parsing(&mut self) -> Vec<Series> {
        let wanted_props = vec![20, 21];
        match self.get_cache_if_exists() {
            Some(mut cache) => {
                println!("Using cache");
                // Bytes where our wanted ticks start
                let mut wanted_bytes = self.get_wanted_bytes(&mut cache);
                for prop in &wanted_props {
                    wanted_bytes.extend(cache.find_delta_ticks(5, *prop));
                }

                self.parse_bytes(wanted_bytes);

                let jobresults = self.compute_jobs(&mut cache, &wanted_props);

                // println!("{:?}", jobresults);
                jobresults
            }
            None => {
                println!("No cache found");

                // Empty vec == parse entire demo
                self.parse_bytes(vec![]);
                let jobresults = self.compute_jobs_no_cache();
                let cache_path = self.get_cache_path();

                let mut wc = WriteCache::new(
                    &cache_path,
                    jobresults,
                    self.state.dt_started_at,
                    self.state.ge_map_started_at,
                );
                wc.write_all_caches();
                vec![]
            }
        }
    }
    pub fn parse_bytes(&mut self, wanted_bytes: Vec<u64>) -> Vec<MsgBluePrint> {
        // Todo dt map idx mutability
        let mut v = vec![];
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

    pub fn compute_tasks(&mut self, tasks: &Vec<MsgBluePrint>) -> Vec<JobResult> {
        for task in tasks {
            if task.msg == 30 {
                self.parse_game_event_map(task);
            }
        }
        tasks
            .into_iter()
            .map(|t| {
                Parser::msg_handler(
                    &t,
                    &self.bytes,
                    &self.maps.serverclass_map,
                    &self.maps.event_map.as_ref().unwrap(),
                    &self.state.stringtables,
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
    pub fn compute_jobs_no_cache(&mut self) -> Vec<JobResult> {
        let tasks = self.tasks.clone();
        // Special msg that is needed for parsing game events.
        // Event comes one time per demo sometime in the beginning
        let before = Instant::now();
        for task in &tasks {
            if task.msg == 30 {
                self.parse_game_event_map(task);
            }
        }
        let results: Vec<JobResult> = self.compute_tasks(&tasks);

        use ndarray::Array3;
        let total_ticks = self.settings.playback_frames * 2;
        let mut df = Array3::<f32>::zeros((12, self.settings.wanted_props.len(), total_ticks));
        // let z = self.get_raw_df(&results, &mut df, total_ticks, );
        // println!("{:?}", z);
        results
    }

    pub fn compute_jobs(&mut self, cache: &mut ReadCache, wanted_props: &Vec<u32>) -> Vec<Series> {
        let tasks = self.tasks.clone();
        // Special msg that is needed for parsing game events.
        // Event comes one time per demo sometime in the beginning
        let before = Instant::now();
        for task in &tasks {
            if task.msg == 30 {
                self.state.ge_map_started_at = (task.start_idx - 6) as u64;
                self.parse_game_event_map(task);
            }
        }
        let results: Vec<JobResult> = self.compute_tasks(&tasks);

        let p = Players::new(&results);
        let mut events = cache.get_game_event_jobs(&results, &p);
        let need_to_parse_bytes = cache.get_event_deltas(21, &mut events);

        // HERERERERERE
        let byte_readers = self.get_byte_readers(need_to_parse_bytes);
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

        let tasks = self.tasks.clone();
        let results: Vec<JobResult> = self.compute_tasks(&tasks);

        let v: Vec<i32> = (20000..50000).collect();
        let mut ss = vec![];
        for p in wanted_props {
            let out = self.functional_searcher(&results, *p as i32, 5, v.clone());
            let s = Series::new("out", out);
            ss.push(s);
        }
        let before = Instant::now();

        println!("Searcher took: {:2?}", before.elapsed());

        use ndarray::Array3;
        let total_ticks = self.settings.playback_frames * 2;
        let mut df = Array3::<f32>::zeros((12, 2, 2));
        let events = self.get_raw_df(&results, &mut df, total_ticks, &p);

        ss
        //println!("{:?}", z);
        //println!("Took {:2?}", before.elapsed());
    }

    pub fn filter_jobs_by_pidx_entid(
        &self,
        results: &Vec<JobResult>,
        entid: i32,
        pidx: i32,
    ) -> Vec<(f32, i32)> {
        /*
        Filters the raw parser outputs into form:
        Vec<Val, Tick>
        That can then be binary searched.
        */
        let mut v = vec![];
        for x in results {
            if let JobResult::PacketEntities(pe) = x {
                v.push(pe);
            }
        }
        v.into_par_iter()
            .flat_map(|x| self.matcher(x, pidx, entid))
            .collect()
    }

    pub fn functional_searcher(
        &self,
        results: &Vec<JobResult>,
        pidx: i32,
        entid: i32,
        ticks: Vec<i32>,
    ) -> Vec<f32> {
        let filtered = self.filter_jobs_by_pidx_entid(results, entid, pidx);

        let mut output = vec![];
        for tick in ticks {
            let idx = filtered.binary_search_by(|segment| segment.1.partial_cmp(&tick).unwrap());
            let p = match idx {
                Ok(i) => filtered[i],
                Err(i) => filtered[i],
            };
            output.push(p.0);
            //println!("{} {:?}", tick, p)
        }
        output
    }
    #[inline(always)]
    pub fn matcher(&self, pe: &PacketEntsOutput, pidx: i32, entid: i32) -> Option<(f32, i32)> {
        for x in &pe.data {
            if x.ent_id == entid && x.prop_inx == pidx {
                match x.data {
                    PropData::F32(f) => {
                        return Some((f, pe.tick));
                    }
                    _ => {}
                }
            }
        }
        None
    }

    pub fn msg_handler(
        blueprint: &MsgBluePrint,
        bytes: &Mmap,
        serverclass_map: &HashMap<u16, ServerClass, RandomState>,
        game_events_map: &HashMap<i32, Descriptor_t, RandomState>,
        stringtables: &Vec<StringTable>,
    ) -> JobResult {
        let wanted_event = "player_death";
        // println!("{:?} {}", blueprint.tick, blueprint.msg);

        match blueprint.msg {
            26 => parse_packet_entities(blueprint, bytes, serverclass_map),
            25 => Parser::parse_game_events(blueprint, bytes, game_events_map, wanted_event),
            12 => Parser::create_string_table(blueprint, bytes),
            13 => Parser::update_string_table_msg(blueprint, bytes),
            _ => JobResult::None,
        }
    }
}
