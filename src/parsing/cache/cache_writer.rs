use crate::parsing::demo_parsing::*;
use crate::parsing::parser::JobResult;
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
    pub fn to_str_name_player_prop(
        &mut self,
        sv_cls_map: &HashMap<u16, ServerClass>,
        idx: i32,
    ) -> String {
        let player_props = &sv_cls_map.get(&40).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        "player_".to_string() + &prop.table.to_owned() + "." + &prop.name.to_owned()
    }
    pub fn to_str_name_team_prop(
        &mut self,
        sv_cls_map: &HashMap<u16, ServerClass>,
        idx: i32,
    ) -> String {
        let player_props = &sv_cls_map.get(&43).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        "team_".to_string() + &prop.table.to_owned() + "." + &prop.name.to_owned()
    }
    pub fn to_str_name_manager_prop(
        &mut self,
        sv_cls_map: &HashMap<u16, ServerClass>,
        idx: i32,
    ) -> String {
        let player_props = &sv_cls_map.get(&41).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        "manager_".to_string() + &prop.table.to_owned() + "." + &prop.name.to_owned()
    }
    pub fn to_str_name_rules_prop(
        &mut self,
        sv_cls_map: &HashMap<u16, ServerClass>,
        idx: i32,
    ) -> String {
        let player_props = &sv_cls_map.get(&39).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        "rules_".to_string() + &prop.table.to_owned() + "." + &prop.name.to_owned()
    }

    fn write_bytes_to_zip(&self, data: &Vec<(usize, i32, i32)>) {}

    pub fn write_packet_ents(&mut self, sv_cls_map: &HashMap<u16, ServerClass>) {
        let forbidden = vec![0, 1, 2, 37, 103, 93, 59, 58, 1343, 1297, 40, 41, 26, 27];

        let mut player_props = vec![];
        let mut other_props = vec![];

        for p in &self.packet_ents {
            for x in &p.data {
                if x.ent_id < 64 && x.ent_id > 0 {
                    if !forbidden.contains(&x.prop_inx) {
                        player_props.push((p.byte, x.prop_inx, x.ent_id, p.tick))
                    }
                } else {
                    other_props.push((p.byte, x.prop_inx, x.ent_id, p.tick))
                }
            }
        }
        println!("PL {}", player_props.len());
        println!("PL {}", other_props.len());

        let m = player_props.iter().into_group_map_by(|x| x.1);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Zstd);

        /*
        Write data per prop. Also use SoA form for ~3x size reduction.
        */

        for idx in 0..11000 {
            match m.get(&idx) {
                Some(g) => {
                    let prop_str_name = if idx == 10000 {
                        "player_m_vecOrigin_X".to_string()
                    } else if idx == 10001 {
                        "player_m_vecOrigin_Y".to_string()
                    } else {
                        self.to_str_name_player_prop(sv_cls_map, idx)
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
        let mut teams = vec![];
        let mut manager = vec![];
        let mut rules = vec![];

        for field in &other_props {
            match field.2 {
                // TEAM
                65 => teams.push(field),
                66 => teams.push(field),
                67 => teams.push(field),
                68 => teams.push(field),
                69 => teams.push(field),
                // MANAGER
                70 => manager.push(field),
                // RULES
                71 => rules.push(field),
                _ => {}
            }
        }
        let grouped_by_pidx = teams.iter().into_group_map_by(|x| x.1);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Zstd);
        for (pidx, data) in grouped_by_pidx {
            let str_name = self.to_str_name_team_prop(sv_cls_map, pidx);

            self.zip.start_file(str_name, options).unwrap();
            let mut byt = vec![];

            byt.extend(data.len().to_le_bytes());
            for t in &data {
                byt.extend(t.0.to_le_bytes());
            }
            for t in &data {
                byt.extend(t.3.to_le_bytes());
            }
            for t in &data {
                byt.extend(t.2.to_le_bytes());
            }

            self.zip.write_all(&byt).unwrap();
        }
        /*
        self.zip.start_file("other_props", options).unwrap();

        let mut byt = vec![];
        for field in &other_props {
            byt.extend(field.0.to_le_bytes());
        }
        for field in &other_props {
            byt.extend(field.3.to_le_bytes());
        }
        for field in &other_props {
            byt.extend(field.2.to_le_bytes());
        }
        self.zip.write_all(&byt).unwrap();
        */
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
