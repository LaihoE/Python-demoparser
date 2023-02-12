use crate::parsing::demo_parsing::*;
use crate::parsing::parser::JobResult;
use ahash::HashMap;
use itertools::Itertools;
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
    pub packet_ents: Vec<Vec<EntityIndicies>>,
    pub dt_start: u64,
    pub ge_start: u64,
    pub zip: ZipWriter<File>,
}

struct CacheEntry {
    byte: u64,
    tick: i32,
    pidx: i32,
    entid: u32,
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
        "player@".to_string() + &prop.table.to_owned() + "." + &prop.name.to_owned()
    }
    pub fn to_str_name_team_prop(
        &mut self,
        sv_cls_map: &HashMap<u16, ServerClass>,
        idx: i32,
    ) -> String {
        let player_props = &sv_cls_map.get(&43).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        "team@".to_string() + &prop.table.to_owned() + "." + &prop.name.to_owned()
    }
    pub fn to_str_name_manager_prop(
        &mut self,
        sv_cls_map: &HashMap<u16, ServerClass>,
        idx: i32,
    ) -> String {
        let player_props = &sv_cls_map.get(&41).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        "manager@".to_string() + &prop.table.to_owned() + "." + &prop.name.to_owned()
    }
    pub fn to_str_name_rules_prop(
        &mut self,
        sv_cls_map: &HashMap<u16, ServerClass>,
        idx: i32,
    ) -> String {
        let player_props = &sv_cls_map.get(&39).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        "rules@".to_string() + &prop.table.to_owned() + "." + &prop.name.to_owned()
    }

    fn write_bytes_to_zip(&mut self, data: &Vec<&CacheEntry>, name: &String) {
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Zstd);
        self.zip.start_file(name, options).unwrap();
        let mut byt = vec![];
        byt.extend(data.len().to_le_bytes());
        for t in data {
            byt.extend(t.byte.to_le_bytes());
        }
        for t in data {
            byt.extend(t.tick.to_le_bytes());
        }
        for t in data {
            byt.extend(t.entid.to_le_bytes());
        }
        self.zip.write_all(&byt).unwrap();
    }

    pub fn write_packet_ents(&mut self, sv_cls_map: &HashMap<u16, ServerClass>) {
        let forbidden = vec![0, 1, 2, 37, 103, 93, 59, 58, 1343, 1297, 40, 41, 26, 27];

        let mut player_props = vec![];
        let mut other_props = vec![];

        for indicies in &self.packet_ents {
            for this_ents_indicies in indicies {
                if this_ents_indicies.entid < 64 && this_ents_indicies.entid > 0 {
                    for index in &this_ents_indicies.prop_indicies {
                        if !forbidden.contains(&index) {
                            player_props.push(CacheEntry {
                                byte: this_ents_indicies.byte,
                                pidx: *index,
                                entid: this_ents_indicies.entid as u32,
                                tick: this_ents_indicies.tick,
                            })
                        }
                    }
                } else {
                    for index in &this_ents_indicies.prop_indicies {
                        other_props.push(CacheEntry {
                            byte: this_ents_indicies.byte,
                            pidx: *index,
                            entid: this_ents_indicies.entid as u32,
                            tick: this_ents_indicies.tick,
                        })
                    }
                }
            }
        }

        let group_by_pidx = player_props.iter().into_group_map_by(|x| x.pidx);

        /*
        Write data per prop. Also use SoA form for ~3x size reduction.
        */

        for (pidx, data) in &group_by_pidx {
            let prop_str_name = if *pidx == 10000 {
                "player@m_vecOrigin_X".to_string()
            } else if *pidx == 10001 {
                "player@m_vecOrigin_Y".to_string()
            } else {
                self.to_str_name_player_prop(sv_cls_map, *pidx)
            };
            self.write_bytes_to_zip(data, &prop_str_name);
        }

        let mut teams = vec![];
        let mut manager = vec![];
        let mut rules = vec![];

        for field in other_props {
            match field.entid {
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
        self.write_others(teams, sv_cls_map, "teams");
        self.write_others(manager, sv_cls_map, "manager");
        self.write_others(rules, sv_cls_map, "rules");
    }
    fn write_others(
        &mut self,
        data: Vec<CacheEntry>,
        sv_cls_map: &HashMap<u16, ServerClass>,
        write_type: &str,
    ) {
        let grouped_by_pidx = data.iter().into_group_map_by(|x| x.pidx);
        for (pidx, data) in grouped_by_pidx {
            let str_name = match write_type {
                "teams" => self.to_str_name_team_prop(sv_cls_map, pidx),
                "manager" => self.to_str_name_manager_prop(sv_cls_map, pidx),
                "rules" => self.to_str_name_rules_prop(sv_cls_map, pidx),
                _ => panic!("unkown write type"),
            };
            self.write_bytes_to_zip(&data, &str_name);
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
    ) -> (Vec<GameEvent>, Vec<UserInfo>, Vec<Vec<EntityIndicies>>) {
        let mut game_events = vec![];
        let mut string_tables = vec![];
        let mut packet_ents = vec![];

        for jobresult in jobresults {
            match jobresult {
                JobResult::GameEvents(ge) => game_events.extend(ge),
                JobResult::PacketEntitiesIndicies(pe) => packet_ents.push(pe),
                JobResult::StringTables(st) => string_tables.extend(st),
                _ => {}
            }
        }
        (game_events, string_tables, packet_ents)
    }
}
