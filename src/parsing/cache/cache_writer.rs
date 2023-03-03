use crate::parsing::cache::ReadCache;
use crate::parsing::demo_parsing::EidClsHistoryEntry;
use crate::parsing::demo_parsing::GameEventHistory;
use crate::parsing::demo_parsing::ServerClass;
use crate::parsing::demo_parsing::StringTableHistory;
use crate::parsing::utils::CACHE_ID_MAP;
use ahash::HashMap;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use hdf5::{File, Group};
use ndarray::arr1;
use ndarray::Array1;
use serde::Deserialize;
use serde::Serialize;
use serde_json::to_string;
use std::fs;
use std::io::prelude::*;

pub const HASH_BYTE_LENGTH: usize = 10000;
pub const EID_CLS_MAP_ID: i32 = 99999;
pub const GAME_EVENT_ID: i32 = -5;
pub const STRING_TABLE_ID: i32 = -6;
pub const AMMO_ID: i32 = -42;
pub const ITEMDEF_ID: i32 = -88;
pub const ACTIVE_WEAPON_ID: i32 = 8;

#[derive(Debug)]
pub struct WriteCache {
    pub path: String,
    pub index: Vec<IndexEntry>,
    pub buffer: Vec<u8>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct IndexEntry {
    pub byte_start_at: i32,
    pub byte_end_at: i32,
    pub id: i32,
}

impl WriteCache {
    pub fn new(bytes: &[u8]) -> Self {
        let cache_path = ReadCache::get_cache_path(bytes);
        WriteCache {
            path: cache_path,
            index: vec![],
            buffer: vec![],
        }
    }
    pub fn compress_bytes(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::fast());
        e.write_all(bytes).unwrap();
        e.finish().unwrap()
    }
    pub fn append_to_buffer(&mut self, bytes: &[u8], id: i32) {
        let compressed = self.compress_bytes(bytes);
        let entry = IndexEntry {
            byte_start_at: self.buffer.len() as i32,
            byte_end_at: (self.buffer.len() + compressed.len()) as i32,
            id: id,
        };
        self.index.push(entry);
        self.buffer.extend(compressed);
    }
    pub fn flush(&mut self) {
        let mut index_bytes = vec![];
        let index_starts_at_byte = self.buffer.len();
        for entry in &self.index {
            index_bytes.extend(entry.byte_start_at.to_le_bytes());
            index_bytes.extend(entry.byte_end_at.to_le_bytes());
            index_bytes.extend(entry.id.to_le_bytes());
        }
        self.buffer.extend(index_bytes);
        self.buffer.extend(index_starts_at_byte.to_le_bytes());
        fs::write(&self.path, &self.buffer).unwrap();
    }
    pub fn write_dt_ge_map(&mut self, ge_start_at: u64, dt_start_at: u64) {
        let mut bytes = vec![];
        bytes.extend(dt_start_at.to_le_bytes());
        bytes.extend(ge_start_at.to_le_bytes());
        self.append_to_buffer(&bytes, -2);
    }

    pub fn write_eid_cls_map(&mut self, eid_cls_map: &Vec<EidClsHistoryEntry>) {
        let mut bytes = vec![];
        //for (k, v) in eid_cls_map

        bytes.extend(eid_cls_map.iter().flat_map(|x| x.cls_id.to_le_bytes()));
        bytes.extend(eid_cls_map.iter().flat_map(|x| x.eid.to_le_bytes()));
        bytes.extend(eid_cls_map.iter().flat_map(|x| x.tick.to_le_bytes()));
        bytes.extend(eid_cls_map.iter().flat_map(|x| x.byte.to_le_bytes()));
        self.append_to_buffer(&bytes, EID_CLS_MAP_ID);
    }

    pub fn write_game_events(&mut self, game_events: &Vec<GameEventHistory>) {
        let mut bytes = vec![];
        bytes.extend(game_events.iter().flat_map(|x| x.byte.to_le_bytes()));
        bytes.extend(game_events.iter().flat_map(|x| x.tick.to_le_bytes()));
        bytes.extend(game_events.iter().flat_map(|x| x.id.to_le_bytes()));
        self.append_to_buffer(&bytes, GAME_EVENT_ID)
    }
    pub fn write_stringtables(&mut self, stringtables: &Vec<StringTableHistory>) {
        let mut bytes = vec![];
        bytes.extend(stringtables.iter().flat_map(|x| x.byte.to_le_bytes()));
        self.append_to_buffer(&bytes, STRING_TABLE_ID)
    }

    pub fn write_packet_ents(
        &mut self,
        packet_ents: &HashMap<u32, HashMap<u32, Vec<[i32; 3]>>>,
        serverclass_map: &HashMap<u16, ServerClass>,
    ) {
        let mut ammo_ticks = vec![];
        let mut ammo_bytes = vec![];
        let mut ammo_entids = vec![];

        let mut def_bytes = vec![];
        let mut def_ticks = vec![];
        let mut def_entids = vec![];

        for (cls_id, inner) in packet_ents {
            if let Some(serverclass) = &serverclass_map.get(&(*cls_id as u16)) {
                for (pidx, v) in inner {
                    if let Some(prop) = serverclass.props.get(*pidx as usize) {
                        let mut temp_arr: Vec<u8> = vec![];
                        temp_arr.extend(v.iter().flat_map(|x| x[0].to_le_bytes()));
                        temp_arr.extend(v.iter().flat_map(|x| x[1].to_le_bytes()));
                        temp_arr.extend(v.iter().flat_map(|x| x[2].to_le_bytes()));

                        let prop_name =
                            serverclass.dt.to_string() + "-" + &prop.table + "-" + &prop.name;

                        match prop.name.as_str() {
                            "m_iClip1" => {
                                //println!("CLIP");
                                ammo_bytes.extend(v.iter().flat_map(|x| x[0].to_le_bytes()));
                                ammo_ticks.extend(v.iter().flat_map(|x| x[1].to_le_bytes()));
                                ammo_entids.extend(v.iter().flat_map(|x| 0_i32.to_le_bytes()));
                            }
                            "m_iItemDefinitionIndex" => {
                                //println!("HRERE");
                                def_bytes.extend(v.iter().flat_map(|x| x[0].to_le_bytes()));
                                def_ticks.extend(v.iter().flat_map(|x| x[1].to_le_bytes()));
                                def_entids.extend(v.iter().flat_map(|x| 0_i32.to_le_bytes()));
                            }
                            _ => {
                                if serverclass.id == 40 {
                                    let prop_id = match CACHE_ID_MAP.get(&prop.name) {
                                        Some(k) => k,
                                        None => &-69,
                                    };
                                    self.append_to_buffer(&temp_arr, *prop_id)
                                }
                            }
                        }
                    }
                }
            }
        }
        ammo_bytes.extend(ammo_ticks);
        ammo_bytes.extend(ammo_entids);
        def_bytes.extend(def_ticks);
        def_bytes.extend(def_entids);
        self.append_to_buffer(&def_bytes, ITEMDEF_ID);
        self.append_to_buffer(&ammo_bytes, AMMO_ID);
    }
}
