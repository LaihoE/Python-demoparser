use crate::parsing::demo_parsing::*;
use crate::parsing::parser::JobResult;
use ahash::HashMap;
use itertools::Itertools;
use std::fs;
use std::fs::File;
use std::io::Write;
use zip::write::FileOptions;
use zip::ZipWriter;

pub struct WriteCache {
    pub game_events: Vec<GameEvent>,
    pub string_tables: Vec<UserInfo>,
    pub packet_ents: Vec<Vec<EntityIndicies>>,
    pub dt_start: u64,
    pub ge_start: u64,
    pub zip: ZipWriter<File>,
    pub zip_options: FileOptions,
}
#[derive(Debug)]
struct CacheEntry {
    byte: u64,
    pidx: i32,
    entid: u32,
}

pub const TEAM_CLSID: u16 = 43;
pub const MANAGER_CLSID: u16 = 41;
pub const RULES_CLSID: u16 = 39;
pub const PLAYER_CLSID: u16 = 40;
pub const PLAYER_MAX_ENTID: i32 = 64;

impl WriteCache {
    pub fn new(path: &String, jobresults: Vec<JobResult>, dt_start: u64, ge_start: u64) -> Self {
        let (game_events, string_tables, packet_ents) = WriteCache::filter_per_result(jobresults);

        let file = fs::File::create(path.to_owned()).unwrap();
        let zip = zip::ZipWriter::new(file);

        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Zstd)
            .compression_level(Some(10));

        WriteCache {
            game_events: game_events,
            string_tables: string_tables,
            packet_ents: packet_ents,
            dt_start: dt_start,
            ge_start: ge_start,
            zip: zip,
            zip_options: options,
        }
    }
    pub fn write_all_caches(&mut self, sv_cls_map: &HashMap<u16, ServerClass>) {
        self.write_packet_ents(sv_cls_map);
        self.write_game_events();
        self.write_string_tables();
        self.write_maps();
    }
    pub fn write_packet_ents(&mut self, sv_cls_map: &HashMap<u16, ServerClass>) {
        let (player_props, other_props, hm, inverse_hm) = self.split_props_gen_idx_map();
        self.write_tick_map(&inverse_hm);
        self.write_player_props(player_props, &hm, sv_cls_map);
        self.write_other_props(other_props, &hm, sv_cls_map);
    }

    pub fn idx_to_str_name(
        &mut self,
        sv_cls_map: &HashMap<u16, ServerClass>,
        idx: i32,
        write_type: &str,
        cls_id: u16,
    ) -> String {
        let player_props = &sv_cls_map.get(&cls_id).unwrap().props;
        let prop = player_props.get(idx as usize).unwrap();
        write_type.to_string() + "@" + &prop.table + "." + &prop.name
    }

    fn write_bytes_to_zip(
        &mut self,
        data: &mut Vec<&CacheEntry>,
        name: &String,
        hm: &HashMap<u32, i32>,
    ) {
        self.zip.start_file(name, self.zip_options).unwrap();
        let per_prop = data.iter().group_by(|x| x.byte);

        let mut vals: Vec<(u64, i16)> = vec![];
        for (k, v) in &per_prop {
            let mut mask: i16 = 0;

            for e in v {
                if e.entid < 16 {
                    mask |= 1 << e.entid;
                }
            }
            vals.push((k, mask));
        }

        vals.sort_unstable_by_key(|x| x.0);
        for i in &vals {
            //println!("{} {} {}", hm[&(i.0 as u32)], i.1, name);
        }
        let mut bytes: Vec<u8> = vec![];

        bytes.extend(vals.len().to_le_bytes());
        bytes.extend(vals.iter().flat_map(|e| hm[&(e.0 as u32)].to_le_bytes()));
        bytes.extend(vals.iter().flat_map(|e| e.1.to_le_bytes()));

        self.zip.write_all(&bytes).unwrap();
    }

    fn push_cache_entry(
        forbidden: &Vec<i32>,
        entindc: &EntityIndicies,
        out_vec: &mut Vec<CacheEntry>,
    ) {
        for idx in &entindc.prop_indicies {
            if !forbidden.contains(&idx) || entindc.entid > PLAYER_MAX_ENTID {
                out_vec.push(CacheEntry {
                    byte: entindc.byte,
                    pidx: *idx,
                    entid: entindc.entid as u32,
                })
            }
        }
    }
    fn insert_tick_maps(
        hm: &mut HashMap<u32, i32>,
        inverse_hm: &mut HashMap<i32, (u64, i32)>,
        entindc: &EntityIndicies,
        tick_map_idx: &mut i32,
    ) {
        if !hm.contains_key(&(entindc.byte as u32)) {
            hm.insert(entindc.byte as u32, *tick_map_idx as i32);
            inverse_hm.insert(*tick_map_idx as i32, (entindc.byte, entindc.tick));
            *tick_map_idx += 1;
        }
    }
    fn split_props_gen_idx_map(
        &mut self,
    ) -> (
        Vec<CacheEntry>,
        Vec<CacheEntry>,
        HashMap<u32, i32>,
        HashMap<i32, (u64, i32)>,
    ) {
        // TODO, they seem constant across demos... only for player props
        let forbidden = vec![0, 1, 2, 37, 103, 93, 59, 58, 40, 41, 26, 27];

        let mut player_props = vec![];
        let mut other_props = vec![];
        let mut hm: HashMap<u32, i32> = HashMap::default();
        let mut inverse_hm = HashMap::default();
        let mut tick_map_idx = 0;

        for indicies in &self.packet_ents {
            for this_ents_indicies in indicies {
                // PLAYERS HAVE ENTID < 64
                let out_vec = match this_ents_indicies.entid < PLAYER_MAX_ENTID {
                    true => &mut player_props,
                    false => &mut other_props,
                };
                WriteCache::push_cache_entry(&forbidden, this_ents_indicies, out_vec);
                WriteCache::insert_tick_maps(
                    &mut hm,
                    &mut inverse_hm,
                    this_ents_indicies,
                    &mut tick_map_idx,
                );
            }
        }
        (player_props, other_props, hm, inverse_hm)
    }
    fn write_player_props(
        &mut self,
        player_props: Vec<CacheEntry>,
        hm: &HashMap<u32, i32>,
        sv_cls_map: &HashMap<u16, ServerClass>,
    ) {
        let mut group_by_pidx = player_props.iter().into_group_map_by(|x| x.pidx);

        for (pidx, data) in &mut group_by_pidx {
            let prop_str_name = if *pidx == 10000 {
                "player@m_vecOrigin_X".to_string()
            } else if *pidx == 10001 {
                "player@m_vecOrigin_Y".to_string()
            } else {
                self.idx_to_str_name(sv_cls_map, *pidx, "player", PLAYER_CLSID)
            };
            self.write_bytes_to_zip(data, &prop_str_name, &hm);
        }
    }
    pub fn write_maps(&mut self) {
        // Writes positions where Game event map and dt map start.
        // These are needed for parsing the rest of the demo so
        // these have to be parsed always.
        self.zip.start_file("maps", self.zip_options).unwrap();

        let mut bytes = vec![];
        bytes.extend(self.ge_start.to_le_bytes());
        bytes.extend(self.dt_start.to_le_bytes());
        self.zip.write_all(&bytes).unwrap();
    }
    fn write_other_props(
        &mut self,
        other_props: Vec<CacheEntry>,
        hm: &HashMap<u32, i32>,
        sv_cls_map: &HashMap<u16, ServerClass>,
    ) {
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
        self.write_others(teams, sv_cls_map, "teams", &hm);
        self.write_others(manager, sv_cls_map, "manager", &hm);
        self.write_others(rules, sv_cls_map, "rules", &hm);
    }

    fn write_others(
        &mut self,
        data: Vec<CacheEntry>,
        sv_cls_map: &HashMap<u16, ServerClass>,
        write_type: &str,
        hm: &HashMap<u32, i32>,
    ) {
        let mut grouped_by_pidx = data.iter().into_group_map_by(|x| x.pidx);
        for (pidx, data) in &mut grouped_by_pidx {
            let str_name = match write_type {
                "teams" => self.idx_to_str_name(sv_cls_map, *pidx, write_type, TEAM_CLSID),
                "manager" => self.idx_to_str_name(sv_cls_map, *pidx, write_type, MANAGER_CLSID),
                "rules" => self.idx_to_str_name(sv_cls_map, *pidx, write_type, RULES_CLSID),
                _ => panic!("unkown write type"),
            };
            self.write_bytes_to_zip(data, &str_name, &hm);
        }
    }

    pub fn filter_per_result(
        jobresults: Vec<JobResult>,
    ) -> (Vec<GameEvent>, Vec<UserInfo>, Vec<Vec<EntityIndicies>>) {
        let mut game_events = vec![];
        let mut string_tables = vec![];
        let mut packet_ents = vec![];

        for jobresult in jobresults {
            match jobresult {
                JobResult::GameEvents(ge) => game_events.push(ge),
                JobResult::PacketEntitiesIndicies(pe) => packet_ents.push(pe),
                JobResult::StringTables(st) => string_tables.extend(st),
                _ => {}
            }
        }
        (game_events, string_tables, packet_ents)
    }
    fn write_tick_map(&mut self, inverse_hm: &HashMap<i32, (u64, i32)>) {
        // Writes hm that maps idx --> (tick, byte) pairs.
        // This is done to compress better ie. small integer vs (integer + u64)

        self.zip
            .start_file("tick_mapping", self.zip_options)
            .unwrap();
        let mut bytes = vec![];
        bytes.extend(inverse_hm.len().to_le_bytes());
        bytes.extend(inverse_hm.keys().flat_map(|x| x.to_le_bytes()));
        bytes.extend(inverse_hm.iter().flat_map(|(_, v)| v.0.to_le_bytes()));
        bytes.extend(inverse_hm.iter().flat_map(|(_, v)| v.1.to_le_bytes()));

        self.zip.write_all(&bytes).unwrap();
    }
    pub fn write_string_tables(&mut self) {
        self.zip
            .start_file("string_tables", self.zip_options)
            .unwrap();
        let mut byt = vec![];
        byt.extend(self.string_tables.len().to_le_bytes());

        for st in &self.string_tables {
            byt.extend(st.byte.to_le_bytes());
        }
        self.zip.write_all(&byt).unwrap();
    }

    pub fn write_game_events(&mut self) {
        self.zip
            .start_file("game_events", self.zip_options)
            .unwrap();

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
}
