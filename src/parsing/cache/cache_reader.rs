use crate::parsing::cache::IndexEntry;
use crate::parsing::cache::GAME_EVENT_ID;
use crate::parsing::demo_parsing::EidClsHistoryEntry;
use ahash::HashMap;
use ahash::HashSet;
use flate2::bufread::ZlibDecoder;
use itertools::izip;
use itertools::Itertools;
use memmap2::Mmap;
use memmap2::MmapOptions;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::AMMO_ID;
use super::ITEMDEF_ID;
use super::STRING_TABLE_ID;
use crate::parsing::cache::ACTIVE_WEAPON_ID;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Delta {
    pub byte: i32,
    pub entid: i32,
    pub tick: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct GameEventIdx {
    pub byte: i32,
    pub tick: i32,
    pub id: i32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Stringtables {
    pub byte: u64,
}

pub struct ReadCache {
    pub deltas: HashMap<i32, Vec<Delta>>,
    pub path: String,
    pub bytes: Mmap,
    pub index: HashMap<i32, IndexEntry>,
    pub game_events: Vec<GameEventIdx>,
}

const HASH_BYTE_LENGTH: usize = 10000;

impl ReadCache {
    pub fn new(path: &str) -> Self {
        let file = File::open(&path).unwrap();
        let map = unsafe { MmapOptions::new().map(&file).unwrap() };

        ReadCache {
            deltas: HashMap::default(),
            path: path.to_string(),
            bytes: map,
            index: HashMap::default(),
            game_events: vec![],
        }
    }
    pub fn get_cache_path(bytes: &[u8]) -> String {
        let file_hash = sha256::digest(&bytes[..HASH_BYTE_LENGTH]);
        let path = "/home/laiho/Documents/cache/".to_owned();
        path + &file_hash + &".h5"
    }
    pub fn get_cache_if_exists(bytes: &[u8]) -> Option<ReadCache> {
        let cache_path = Self::get_cache_path(&bytes);
        match Path::new(&cache_path).exists() {
            true => Some(ReadCache::new(&cache_path)),
            false => None,
        }
    }
    pub fn read_index(&mut self) {
        // Last 8 bytes is the offset
        let index_starts_at = &self.bytes[self.bytes.len() - 8..];
        let index_starts_at = usize::from_le_bytes(index_starts_at.try_into().unwrap());

        for chunk in self.bytes[index_starts_at..(self.bytes.len() - 8)].chunks(12) {
            let entry = IndexEntry {
                byte_start_at: i32::from_le_bytes(chunk[..4].try_into().unwrap()),
                byte_end_at: i32::from_le_bytes(chunk[4..8].try_into().unwrap()),
                id: i32::from_le_bytes(chunk[8..].try_into().unwrap()),
            };
            self.index.insert(entry.id, entry);
        }
    }
    pub fn read_dt_ge_map(&mut self) -> (u64, u64) {
        let decompressed_bytes = self.read_bytes_from_index(-2);
        if decompressed_bytes.len() == 0 {
            return (0, 0);
        }
        let dt_started_at = u64::from_le_bytes(decompressed_bytes[..8].try_into().unwrap());
        let ge_started_at = u64::from_le_bytes(decompressed_bytes[8..].try_into().unwrap());
        (dt_started_at, ge_started_at)
    }
    pub fn read_game_events(&mut self) {
        let decompressed_bytes = self.read_bytes_from_index(GAME_EVENT_ID);
        let number_structs = decompressed_bytes.len() / 12;

        let byte_end_at = number_structs * 4;
        let tick_start_at = byte_end_at;
        let tick_end_at = tick_start_at + (number_structs * 4);
        let id_start_at = tick_end_at;

        let mut bytes_start = vec![];
        let mut ticks = vec![];
        let mut ids = vec![];

        for bytes in decompressed_bytes[..byte_end_at].chunks(4) {
            bytes_start.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[tick_start_at..tick_end_at].chunks(4) {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[id_start_at..].chunks(4) {
            ids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for (byte, tick, entid) in izip!(&bytes_start, &ticks, &ids) {
            self.game_events.push(GameEventIdx {
                byte: *byte,
                tick: *tick,
                id: *entid,
            });
        }
    }
    pub fn game_events_by_id(&mut self, id: i32) -> Vec<u64> {
        self.read_game_events();
        self.game_events
            .iter()
            .filter(|x| x.id == id)
            .map(|x| x.byte as u64)
            .collect_vec()
    }
    pub fn get_event_ticks(&mut self, id: i32) -> Vec<i32> {
        self.read_game_events();

        self.game_events
            .iter()
            .filter(|x| x.id == id)
            .map(|x| x.tick)
            .collect_vec()
    }
    pub fn read_stringtables(&mut self) -> Vec<u64> {
        let decompressed_bytes = self.read_bytes_from_index(STRING_TABLE_ID);
        let mut bytes_out = vec![];
        for bytes in decompressed_bytes.chunks(4) {
            bytes_out.push(i32::from_le_bytes(bytes.try_into().unwrap()) as u64);
        }
        bytes_out
    }

    pub fn filter_delta_ticks_wanted(
        &self,
        deltas: &Vec<&Delta>,
        wanted_ticks: &Vec<i32>,
    ) -> Vec<u64> {
        if deltas.len() == 0 {
            return vec![];
        }
        let mut wanted_bytes = Vec::with_capacity(wanted_ticks.len());
        for wanted_tick in wanted_ticks {
            let idx = deltas.partition_point(|x| x.tick < *wanted_tick);
            if idx > 0 {
                wanted_bytes.push(deltas[idx - 1].byte as u64);
            } else {
                wanted_bytes.push(deltas[0].byte as u64);
            }
        }
        wanted_bytes
    }
    pub fn get_eid_cls_map(&mut self) -> HashMap<i32, Vec<EidClsHistoryEntry>> {
        let decompressed_bytes = self.read_bytes_from_index(99999);

        let number_structs = decompressed_bytes.len() / 16;

        let CLS_SIZE = 4;
        let EID_SIZE = 4;
        let TICK_SIZE = 4;

        let cls_end_at = number_structs * CLS_SIZE;
        let eid_start_at = cls_end_at;
        let eid_end_at = eid_start_at + (number_structs * EID_SIZE);
        let tick_start_at = eid_end_at;
        let tick_end_at = eid_end_at + (number_structs * EID_SIZE);
        let byte_start_at = tick_end_at;

        let mut cls_ids = vec![];
        let mut eids = vec![];
        let mut ticks = vec![];
        let mut bytes_out = vec![];

        for bytes in decompressed_bytes[..cls_end_at].chunks(CLS_SIZE) {
            cls_ids.push(u32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[eid_start_at..eid_end_at].chunks(EID_SIZE) {
            eids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[tick_start_at..tick_end_at].chunks(TICK_SIZE) {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[byte_start_at..].chunks(TICK_SIZE) {
            bytes_out.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }

        let mut eid_history = HashMap::default();
        for (cls_id, entid, tick, byte) in izip!(&cls_ids, &eids, &ticks, &bytes_out) {
            //println!("{} {} {} {}", cls_id, entid, tick, byte);
            eid_history
                .entry(*entid)
                .or_insert(vec![])
                .push(EidClsHistoryEntry {
                    cls_id: *cls_id,
                    eid: *entid,
                    tick: *tick,
                    byte: *byte,
                });
        }

        eid_history
    }

    pub fn decompress_bytes(&mut self, start_byte: i32, end_byte: i32) -> Vec<u8> {
        let bytes = &self.bytes[start_byte as usize..end_byte as usize];
        let mut z = ZlibDecoder::new(&bytes[..]);
        let mut decompressed_bytes = vec![];
        z.read_to_end(&mut decompressed_bytes).unwrap();
        decompressed_bytes
    }
    pub fn read_bytes_from_index(&mut self, id: i32) -> Vec<u8> {
        // Finds offsets for our wanted data
        let entry = match self.index.get(&id) {
            Some(entry) => entry,
            None => return vec![],
        };
        let start_byte = entry.byte_start_at;
        let end_byte = entry.byte_end_at;
        // Return decompressed data at those offsets
        self.decompress_bytes(start_byte, end_byte)
    }
    pub fn read_by_id_others(&mut self, id: i32) -> Vec<u64> {
        let decompressed_bytes = self.read_bytes_from_index(id);
        let number_structs = decompressed_bytes.len() / 12;

        let TICKS_SIZE = 4;
        let BYTES_SIZE = 4;
        let ENTIDS_SIZE = 4;

        let ticks_end_at = number_structs * BYTES_SIZE;
        let bytes_start_at = ticks_end_at;
        let bytes_end_at = bytes_start_at + (number_structs * BYTES_SIZE);
        let entids_start_at = bytes_end_at;

        let mut start_bytes = vec![];
        let mut entids = vec![];
        let mut ticks = vec![];

        for bytes in decompressed_bytes[..ticks_end_at].chunks(TICKS_SIZE) {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[bytes_start_at..bytes_end_at].chunks(BYTES_SIZE) {
            start_bytes.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[entids_start_at..].chunks(ENTIDS_SIZE) {
            entids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        self.deltas.insert(id, vec![]);
        let v = self.deltas.get_mut(&id).unwrap();
        v.reserve(number_structs);

        for (byte, entid, tick) in izip!(&start_bytes, &entids, &ticks) {
            v.push(Delta {
                byte: *byte,
                entid: *entid,
                tick: *tick,
            });
        }
        let all_deltas = &self.deltas[&id];
        all_deltas.iter().map(|x| x.byte as u64).collect_vec()
    }

    pub fn read_by_id_players(&mut self, id: i32, wanted_ticks: &Vec<i32>) -> Vec<u64> {
        let decompressed_bytes = self.read_bytes_from_index(id);
        let number_structs = decompressed_bytes.len() / 12;

        let TICKS_SIZE = 4;
        let BYTES_SIZE = 4;
        let ENTIDS_SIZE = 4;

        let ticks_end_at = number_structs * BYTES_SIZE;
        let bytes_start_at = ticks_end_at;
        let bytes_end_at = bytes_start_at + (number_structs * BYTES_SIZE);
        let entids_start_at = bytes_end_at;

        let mut start_bytes = vec![];
        let mut entids = vec![];
        let mut ticks = vec![];

        for bytes in decompressed_bytes[..ticks_end_at].chunks(TICKS_SIZE) {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[bytes_start_at..bytes_end_at].chunks(BYTES_SIZE) {
            start_bytes.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in decompressed_bytes[entids_start_at..].chunks(ENTIDS_SIZE) {
            entids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        self.deltas.insert(id, vec![]);
        let v = self.deltas.get_mut(&id).unwrap();
        v.reserve(number_structs);

        for (byte, entid, tick) in izip!(&start_bytes, &entids, &ticks) {
            v.push(Delta {
                byte: *byte,
                entid: *entid,
                tick: *tick,
            });
        }
        let mut bytes = HashSet::default();
        let all_deltas = &self.deltas[&id];

        for entid in 1..25 {
            let this_ent_deltas: Vec<&Delta> =
                all_deltas.iter().filter(|x| x.entid == entid).collect_vec();
            let this_bytes = self.filter_delta_ticks_wanted(&this_ent_deltas, wanted_ticks);
            for b in this_bytes {
                bytes.insert(b);
            }
        }
        bytes.iter().map(|x| *x).collect_vec()
    }
    pub fn read_weapons(&mut self) -> Vec<u64> {
        let ids = vec![AMMO_ID, ITEMDEF_ID, ACTIVE_WEAPON_ID];
        let mut out_bytes = vec![];
        for id in ids {
            let decompressed_bytes = self.read_bytes_from_index(id);
            let number_structs = decompressed_bytes.len() / 12;

            let TICKS_SIZE = 4;
            let BYTES_SIZE = 4;
            let ENTIDS_SIZE = 4;

            let ticks_end_at = number_structs * BYTES_SIZE;
            let bytes_start_at = ticks_end_at;
            let bytes_end_at = bytes_start_at + (number_structs * BYTES_SIZE);
            let entids_start_at = bytes_end_at;

            let mut start_bytes = vec![];
            let mut entids = vec![];
            let mut ticks = vec![];

            for bytes in decompressed_bytes[..ticks_end_at].chunks(TICKS_SIZE) {
                ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
            }
            for bytes in decompressed_bytes[bytes_start_at..bytes_end_at].chunks(BYTES_SIZE) {
                start_bytes.push(i32::from_le_bytes(bytes.try_into().unwrap()));
            }
            for bytes in decompressed_bytes[entids_start_at..].chunks(ENTIDS_SIZE) {
                entids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
            }
            out_bytes.extend(start_bytes.iter().map(|x| *x as u64).unique());
        }
        out_bytes
    }
}
