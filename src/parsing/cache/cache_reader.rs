use crate::parsing::cache::IndexEntry;
use crate::parsing::demo_parsing::ServerClass;
use ahash::HashMap;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use flate2::bufread::ZlibDecoder;
use itertools::izip;
use itertools::Itertools;
use memmap2::Mmap;
use ndarray::{arr2, s};
use serde::Deserialize;
use serde::Serialize;
use std::io::Read;
use std::path::Path;
use std::time::Instant;
use zip::result::ZipError;

use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Delta {
    pub byte: u64,
    pub entid: i16,
    pub tick: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GameEventIdx {
    pub byte: u64,
    pub id: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Stringtables {
    pub byte: u64,
}

pub struct ReadCache {
    pub deltas: HashMap<String, Vec<Delta>>,
    pub path: String,
    pub bytes: Mmap,
    pub index: HashMap<i32, IndexEntry>,
}

use memmap2::MmapOptions;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;

const HASH_BYTE_LENGTH: usize = 10000;

impl ReadCache {
    pub fn new(bytes: &[u8]) -> Self {
        let path = ReadCache::get_cache_path(bytes);
        let file = File::open(&path).unwrap();
        let map = unsafe { MmapOptions::new().map(&file).unwrap() };

        ReadCache {
            deltas: HashMap::default(),
            path: path,
            bytes: map,
            index: HashMap::default(),
        }
    }
    pub fn read_index(&mut self) {
        let file = File::open(self.path.clone()).unwrap();
        let index_starts_at = &self.bytes[self.bytes.len() - 8..];
        let index_starts_at = usize::from_le_bytes(index_starts_at.try_into().unwrap());
        let before = Instant::now();
        for chunk in self.bytes[index_starts_at..(self.bytes.len() - 8)].chunks(12) {
            let entry = IndexEntry {
                byte_start_at: i32::from_le_bytes(chunk[..4].try_into().unwrap()),
                byte_end_at: i32::from_le_bytes(chunk[4..8].try_into().unwrap()),
                id: i32::from_le_bytes(chunk[8..].try_into().unwrap()),
            };
            self.index.insert(entry.id, entry);
        }
        println!("Serializing took: {:2?}", before.elapsed());
    }

    pub fn get_cache_path(bytes: &[u8]) -> String {
        let file_hash = sha256::digest(&bytes[..HASH_BYTE_LENGTH]);
        let path = "/home/laiho/Documents/cache/".to_owned();
        path + &file_hash + &".h5"
    }

    pub fn filter_delta_ticks_wanted(
        &self,
        deltas: &Vec<Delta>,
        wanted_ticks: &Vec<i32>,
    ) -> Vec<u64> {
        if deltas.len() == 0 {
            return vec![];
        }
        let mut wanted_bytes = Vec::with_capacity(wanted_ticks.len());
        for wanted_tick in wanted_ticks {
            let idx = deltas.partition_point(|x| x.tick < *wanted_tick);
            if idx > 0 {
                wanted_bytes.push(deltas[idx - 1].byte);
            } else {
                wanted_bytes.push(deltas[0].byte);
            }
        }
        wanted_bytes
    }
    /*
        pub fn read_maps(&mut self) -> (i32, i32) {
            let ds = self.file.dataset(&"/root/maps").unwrap();
            let v = ds.read_1d::<i32>().unwrap();
            let ints = v.to_vec();
            (ints[0], ints[1])
        }
    */

    pub fn read_by_id(&mut self, id: i32) {
        let entry = &self.index[&id];
        let start_byte = entry.byte_start_at;
        let end_byte = entry.byte_end_at;

        let bytes = &self.bytes[start_byte as usize..end_byte as usize];

        let mut z = ZlibDecoder::new(&bytes[..]);
        let mut decompressed_bytes = vec![];
        z.read_to_end(&mut decompressed_bytes).unwrap();

        let total_entries = decompressed_bytes.len() / 12;

        /*
        self.deltas.insert(name.to_string(), vec![]);
        let v = self.deltas.get_mut(&name).unwrap();
        v.reserve(ticks.len());

        for (tick, byte, entid) in izip!(ticks, bytes, entids) {
            v.push(Delta {
                byte: *byte as u64,
                entid: *entid as i16,
                tick: *tick,
            })
        }
        let my = self.deltas.get(&name).unwrap();
        let wanted_ticks = (10000..11000).collect_vec();
        self.filter_delta_ticks_wanted(my, &wanted_ticks)
        */
    }
}
