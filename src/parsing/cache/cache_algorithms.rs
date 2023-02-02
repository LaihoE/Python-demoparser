use crate::parsing::cache::*;
use crate::parsing::data_table::ServerClass;
use crate::parsing::game_events;
use ahash::HashMap;
use itertools::Itertools;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_cbor;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{create_dir, metadata};
use std::io::Read;
use std::io::Write;
use std::{fs, time::Instant};
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::players::Players;

impl ReadCache {
    pub fn find_delta_ticks(
        &mut self,
        userid: u32,
        prop_name: String,
        wanted_ticks: &Vec<i32>,
        players: &Players,
    ) -> Vec<u64> {
        let delta_vec = self.deltas.get(&prop_name).unwrap();
        let wanted_sid = players.uid_to_steamid(userid).unwrap();

        let all_deltas: Vec<(u64, i32, u32)> = delta_vec
            .iter()
            .filter(|x| players.eid_to_sid(x.entid, x.tick) == Some(wanted_sid))
            .map(|x| (x.byte, x.tick, x.entid))
            .collect();

        self.filter_delta_ticks_wanted(&all_deltas, wanted_ticks)
    }

    pub fn filter_delta_ticks_wanted(
        &self,
        temp_ticks: &Vec<(u64, i32, u32)>,
        wanted_ticks: &Vec<i32>,
    ) -> Vec<u64> {
        if temp_ticks.len() == 0 {
            return vec![];
        }

        let mut wanted_bytes = Vec::with_capacity(wanted_ticks.len());
        let mut sorted_ticks = temp_ticks.clone();

        sorted_ticks.sort_by_key(|x| x.1);

        for wanted_tick in wanted_ticks {
            for j in sorted_ticks.windows(2) {
                if j[0].1 <= *wanted_tick && j[1].1 > *wanted_tick {
                    wanted_bytes.push(j[0].0);
                    break;
                }
            }
        }
        wanted_bytes
    }

    pub fn get_stringtables(&self) -> Vec<u64> {
        self.stringtables.iter().map(|s| s.byte).collect()
    }
}
