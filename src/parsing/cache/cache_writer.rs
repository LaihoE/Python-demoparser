use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::PacketEntsOutput;
use crate::parsing::parser::JobResult;
use crate::parsing::stringtables::UserInfo;
use crate::GameEvent;
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
pub struct WriteCache {
    pub game_events: Vec<GameEvent>,
    pub string_tables: Vec<UserInfo>,
    pub packet_ents: Vec<PacketEntsOutput>,
    pub dt_start: u64,
    pub ge_start: u64,
    pub zip: ZipWriter<File>,
}
use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::players::Players;

impl WriteCache {
    pub fn new(path: &String, jobresults: Vec<JobResult>, dt_start: u64, ge_start: u64) -> Self {
        let (game_events, string_tables, packet_ents) = WriteCache::filter_per_result(jobresults);

        let mut file = fs::File::create(path.to_owned()).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        WriteCache {
            game_events: game_events,
            string_tables: string_tables,
            packet_ents: packet_ents,
            dt_start: dt_start,
            ge_start: ge_start,
            zip: zip,
        }
    }
    pub fn write_all_caches(&mut self, sv_cls_map: &HashMap<u16, ServerClass>) {
        self.write_game_events();
        self.write_string_tables();
        self.write_maps();
        self.write_packet_ents(sv_cls_map);
    }

    pub fn write_maps(&mut self) {
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        self.zip.start_file("maps", options).unwrap();

        let mut byt = vec![];
        byt.extend(self.ge_start.to_le_bytes());
        byt.extend(self.dt_start.to_le_bytes());

        self.zip.write_all(&byt).unwrap();
    }
    pub fn to_str_name(&mut self, sv_cls_map: &HashMap<u16, ServerClass>, idx: i32) -> String {
        let player_props = &sv_cls_map.get(&40).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        prop.table.to_owned() + "." + &prop.name.to_owned()
    }

    pub fn write_packet_ents(&mut self, sv_cls_map: &HashMap<u16, ServerClass>) {
        let forbidden = vec![0, 1, 2, 37, 103, 93, 59, 58, 1343, 1297, 40, 41, 26, 27];

        let mut v = vec![];
        for p in &self.packet_ents {
            for x in &p.data {
                if !forbidden.contains(&x.prop_inx) || x.ent_id > 64 {
                    if x.ent_id < 64 && x.ent_id > 0 {
                        v.push((p.byte, x.prop_inx, x.ent_id, p.tick))
                    }
                }
            }
        }

        let m = v.iter().into_group_map_by(|x| x.1);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Zstd);

        /*
        Write data per prop. Also use SoA form for ~3x size reduction.
        */

        for idx in 0..11000 {
            match m.get(&idx) {
                Some(g) => {
                    let prop_str_name = if idx >= 10000 {
                        "m_vecOrigin_X".to_string()
                    } else {
                        self.to_str_name(sv_cls_map, idx)
                    };

                    self.zip.start_file(prop_str_name, options).unwrap();
                    let mut byt = vec![];
                    byt.extend(g.len().to_le_bytes());
                    for t in g {
                        byt.extend(t.0.to_le_bytes());
                    }
                    for t in g {
                        byt.extend(t.3.to_le_bytes());
                    }
                    for t in g {
                        byt.extend(t.2.to_le_bytes());
                    }

                    self.zip.write_all(&byt).unwrap();
                }
                None => {}
            }
        }
    }
    pub fn write_string_tables(&mut self) {
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Zstd);

        self.zip.start_file("string_tables", options).unwrap();
        let mut byt = vec![];
        byt.extend(self.string_tables.len().to_le_bytes());

        for st in &self.string_tables {
            byt.extend(st.byte.to_le_bytes());
        }
        self.zip.write_all(&byt).unwrap();
    }

    pub fn write_game_events(&mut self) {
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Zstd);
        self.zip.start_file("game_events", options).unwrap();

        let mut byt = vec![];
        byt.extend(self.game_events.len().to_le_bytes());

        for ge in &self.game_events {
            byt.extend(ge.byte.to_le_bytes());
        }
        for ge in &self.game_events {
            byt.extend(ge.id.to_le_bytes());
        }
        self.zip.write_all(&byt).unwrap();
    }

    pub fn filter_per_result(
        jobresults: Vec<JobResult>,
    ) -> (Vec<GameEvent>, Vec<UserInfo>, Vec<PacketEntsOutput>) {
        let mut game_events = vec![];
        let mut string_tables = vec![];
        let mut packet_ents = vec![];

        for jobresult in jobresults {
            match jobresult {
                JobResult::GameEvents(ge) => game_events.extend(ge),
                JobResult::PacketEntities(pe) => packet_ents.push(pe),
                JobResult::StringTables(st) => string_tables.extend(st),
                _ => {}
            }
        }
        (game_events, string_tables, packet_ents)
    }
}
