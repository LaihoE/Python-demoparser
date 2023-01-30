use super::cache::ReadCache;
use super::cache::WriteCache;
use super::game_events::GameEvent;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::parse_packet_entities;
use crate::parsing::entities::PacketEntsOutput;
use crate::parsing::parser_settings::*;
use crate::parsing::players::Players;
use crate::parsing::read_bytes::ByteReader;
use crate::parsing::stringtables::UserInfo;
pub use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use itertools::Itertools;
use memmap2::Mmap;
use polars::export::num::NumCast;
use polars::prelude::NamedFrom;
use polars::series::Series;
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
pub struct DataMapping<T> {
    pub data: T,
    pub tick: i32,
    pub entid: i32,
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
        // println!("{}", cache_path);
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
        let mut wanted_bytes = vec![];
        wanted_bytes.push(ge_start as u64);
        wanted_bytes.push(dt_start as u64);

        wanted_bytes.extend(cache.get_stringtables());
        wanted_bytes
    }

    pub fn start_parsing(&mut self) -> Vec<Series> {
        match self.get_cache_if_exists() {
            Some(mut cache) => {
                // println!("Using cache");
                // Bytes where our wanted ticks start
                let wanted_bytes = self.get_wanted_bytes(&mut cache);
                self.parse_bytes(wanted_bytes);

                let jobresults = self.compute_jobs(&mut cache);
                jobresults
            }
            None => {
                // println!("No cache found");

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
        let results: Vec<JobResult> = self.parse_blueprints();
        results
    }

    pub fn compute_jobs(&mut self, cache: &mut ReadCache) -> Vec<Series> {
        // Special msg that is needed for parsing game events.
        // Event comes one time per demo sometime in the beginning

        let results: Vec<JobResult> = self.parse_blueprints();
        let players = Players::new(&results);

        let wanted_props = vec![21];
        let mut wanted_bytes = vec![];
        let uniq_sids = players.get_steamids();

        for prop in &wanted_props {
            cache.read_deltas_by_pidx(*prop);
        }

        for sid in &uniq_sids {
            for prop in &wanted_props {
                let eid = players.sid_to_entid(*sid, 50000).unwrap();
                wanted_bytes.extend(cache.find_delta_ticks(eid, *prop));
            }
        }
        wanted_bytes.sort();
        wanted_bytes.dedup();

        self.parse_bytes(wanted_bytes);
        let results: Vec<JobResult> = self.parse_blueprints();

        let mut ss = vec![];

        let tik = match self.settings.wanted_ticks.len() {
            0 => (0..self.settings.playback_frames as i32).collect(),
            _ => self.settings.wanted_ticks.clone(),
        };

        for p in &wanted_props {
            let (out, labels, ticks) =
                self.functional_searcher(&results, *p as i32, &tik, &players);
            let s = Series::new("yaw", out);
            let ls = Series::new("steamid", labels);
            let ts = Series::new("ticks", ticks);
            ss.push(s);
            ss.push(ls);
            ss.push(ts);
        }
        ss
    }
    //#[inline(always)]
    pub fn filter_jobs_by_pidx(&self, results: &Vec<JobResult>, pidx: i32) -> Vec<(f32, i32, i32)> {
        /*
        Filters the raw parser outputs into form:
        Vec<Val, Tick>
        That can then be binary searched.
        */
        let mut v = vec![];
        for x in results {
            if let JobResult::PacketEntities(pe) = x {
                //println!("{}", pe.tick);
                v.push(pe);
            }
        }

        let mut vector = vec![];

        for pe in v {
            self.matcher(pe, pidx, &mut vector)
        }
        // let x: Vec<(f32, i32, i32)> = v.into_iter().flat_map(|x| self.matcher(x, pidx)).collect();
        vector
    }
    pub fn binary_search_val(
        &self,
        data: &mut Vec<&(f32, i32, i32)>,
        ticks: &Vec<i32>,
    ) -> Vec<f32> {
        let mut output = Vec::with_capacity(ticks.len());

        data.sort_by_key(|x| x.1);

        for tick in ticks {
            let idx = data.binary_search_by(|e_filter| e_filter.1.partial_cmp(&tick).unwrap());

            let p = match idx {
                Ok(i) => {
                    if i == data.len() {
                        data[i - 1]
                    } else {
                        data[i]
                    }
                }
                Err(i) => {
                    if i == data.len() {
                        data[i - 1]
                    } else {
                        data[i]
                    }
                }
            };
            output.push(p.0);
        }

        output
    }

    #[inline(always)]
    pub fn functional_searcher(
        &self,
        results: &Vec<JobResult>,
        pidx: i32,
        ticks: &Vec<i32>,
        players: &Players,
    ) -> (Vec<f32>, Vec<u64>, Vec<i32>) {
        let mut filtered = self.filter_jobs_by_pidx(results, pidx);

        filtered.sort_by_key(|x| x.1);

        // println!("FILTERING TOOK: {:2?}", before.elapsed());

        let grouped_by_entid = filtered
            .iter()
            .into_group_map_by(|x| players.entid_to_sid(x.2 as u32, x.2));

        let mut tasks: Vec<(u64, Vec<&(f32, i32, i32)>)> = vec![];
        let mut labels = vec![];
        let mut out_ticks = vec![];

        for (sid, data) in grouped_by_entid {
            if sid != Some(0) {
                tasks.push((sid.unwrap(), data));
            }
        }
        tasks.sort_by_key(|x| x.0);

        for i in &tasks {
            labels.extend(vec![i.0; ticks.len()]);
            out_ticks.extend(ticks.clone());
        }

        // Vec<Vec<(data, entid, tick)>>    -->  entid -> (data, tick)
        let out: Vec<f32> = tasks
            .iter_mut()
            .flat_map(|(_entid, data)| self.binary_search_val(data, ticks))
            .collect();

        (out, labels, out_ticks)
    }
    #[inline(always)]
    pub fn matcher(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(f32, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                match x.data {
                    PropData::F32(f) => {
                        //println!("found");
                        v.push((f, pe.tick, x.ent_id));
                    }
                    PropData::I32(i) => {
                        // println!("found {}", i);
                        v.push((i as f32, pe.tick, x.ent_id));
                    }
                    _ => {}
                }
            }
        }
        // println!("OUT {} {}", pe.data.len(), pidx);
    }

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
