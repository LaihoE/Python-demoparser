use super::game_events::GameEvent;
use super::utils::TYPEHM;
use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::cache::cache_reader::*;
use crate::parsing::cache::cache_writer::WriteCache;

use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::parse_packet_entities;
use crate::parsing::entities::PacketEntsOutput;
use crate::parsing::parser::*;
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

impl Parser {
    pub fn compute_jobs_no_cache(&mut self) -> Vec<JobResult> {
        let results: Vec<JobResult> = self.parse_blueprints();
        results
    }

    pub fn compute_jobs(&mut self, cache: &mut ReadCache) -> Vec<Series> {
        let tik = match self.settings.wanted_ticks.len() {
            0 => (0..self.settings.playback_frames as i32).collect(),
            _ => self.settings.wanted_ticks.clone(),
        };

        let results_old: Vec<JobResult> = self.parse_blueprints();
        let players = Players::new(&results_old);

        let mut wanted_bytes = vec![];
        let mut wanted_props = self.settings.wanted_props.clone();

        let uniq_uids = players.get_uids();

        // println!("{:?} {:?}", wanted_props, uniq_uids);

        for prop in &wanted_props {
            cache.read_deltas_by_name(prop, &self.maps.serverclass_map);
        }
        for uid in &uniq_uids {
            for prop in &wanted_props {
                wanted_bytes.extend(cache.find_delta_ticks(*uid, prop.to_owned(), &tik, &players));
            }
        }

        wanted_bytes.sort();
        wanted_bytes.dedup();
        self.parse_bytes(wanted_bytes);

        let mut results: Vec<JobResult> = self.parse_blueprints();
        results.extend(results_old);

        let mut ss = vec![];

        for p in &wanted_props {
            let (out, labels, ticks) =
                self.functional_searcher(&results, p.to_owned(), &tik, &players);

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
    pub fn filter_jobs_by_pidx(
        &self,
        results: &Vec<JobResult>,
        prop_idx: i32,
        prop_name: &String,
    ) -> Vec<(f32, i32, i32)> {
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

        let mut vector = vec![];

        let prop_type = TYPEHM.get(&prop_name).unwrap();
        for pe in v {
            match prop_type {
                0 => self.match_int(pe, prop_idx, &mut vector),
                1 => self.match_float(pe, prop_idx, &mut vector),
                // 2 => self.match_str(pe, prop_idx, &mut vector),
                _ => panic!("Unsupported prop type: {}", prop_type),
            }
        }
        vector
    }
    #[inline(always)]
    pub fn match_float(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(f32, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                match x.data {
                    PropData::F32(f) => {
                        v.push((f, pe.tick, x.ent_id));
                    }
                    _ => {}
                }
            }
        }
    }
    #[inline(always)]
    pub fn match_int(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(f32, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                match x.data {
                    PropData::I32(i) => {
                        v.push((i as f32, pe.tick, x.ent_id));
                    }
                    _ => {}
                }
            }
        }
    }
    #[inline(always)]
    pub fn match_str(&self, pe: &PacketEntsOutput, pidx: i32, v: &mut Vec<(String, i32, i32)>) {
        for x in &pe.data {
            if x.prop_inx == pidx && x.ent_id < 64 {
                match &x.data {
                    PropData::String(s) => {
                        v.push((s.to_owned(), pe.tick, x.ent_id));
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn find_wanted_value(
        &self,
        data: &mut Vec<&(f32, i32, i32)>,
        ticks: &Vec<i32>,
    ) -> Vec<f32> {
        /*
        Goes trough wanted data backwards to find correct values
        */

        let mut output = Vec::with_capacity(ticks.len());

        data.sort_by_key(|x| x.1);
        data.reverse();

        for tick in ticks {
            for j in &mut *data {
                if j.1 <= *tick {
                    output.push(j.0);
                    break;
                }
            }
        }
        output
    }

    pub fn str_name_to_idx(&self, str_name: String) -> Option<i32> {
        if str_name == "m_vecOrigin_X" {
            return Some(10000);
        }
        let sv_map = self.maps.serverclass_map.get(&40).unwrap();
        for (idx, prop) in sv_map.props.iter().enumerate() {
            if prop.table.to_owned() + "." + &prop.name.to_owned() == str_name {
                return Some(idx as i32);
            }
        }
        None
    }

    #[inline(always)]
    pub fn functional_searcher(
        &self,
        results: &Vec<JobResult>,
        prop_name: String,
        ticks: &Vec<i32>,
        players: &Players,
    ) -> (Vec<f32>, Vec<u64>, Vec<i32>) {
        // Here we convert string name to idx
        let idx = self.str_name_to_idx(prop_name.clone()).unwrap();

        let mut filtered = self.filter_jobs_by_pidx(results, idx, &prop_name);
        filtered.sort_by_key(|x| x.1);

        let grouped_by_sid = filtered
            .iter()
            .into_group_map_by(|x| players.eid_to_sid(x.2 as u32, x.1));

        let mut tasks: Vec<(u64, Vec<&(f32, i32, i32)>)> = vec![];
        let mut labels = vec![];
        let mut out_ticks = vec![];

        for (sid, data) in grouped_by_sid {
            // println!("{:?} {}", sid, data.len());
            if sid != None && sid != Some(0) {
                tasks.push((sid.unwrap(), data));
            }
        }

        tasks.sort_by_key(|x| x.0);

        for i in &tasks {
            labels.extend(vec![i.0; ticks.len()]);
            out_ticks.extend(ticks.clone());
        }

        let out: Vec<f32> = tasks
            .iter_mut()
            .flat_map(|(_, data)| self.find_wanted_value(data, ticks))
            .collect();

        (out, labels, out_ticks)
    }
}
