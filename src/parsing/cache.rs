use csv::Writer;
use std::fs::{create_dir, metadata};
use std::{fs, time::Instant};

use crate::parsing::parser;
use serde::Deserialize;

use super::{
    entities::PacketEntsOutput,
    game_events::{self, GameEvent},
    parser::JobResult,
    players::Players,
    stringtables::UserInfo,
};
pub struct ReadCache {
    pub deltas: Vec<Delta>,
    pub game_events: Vec<GameEventIdx>,
    pub stringtables: Vec<Stringtables>,
    pub cache_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    byte: u64,
    idx: u32,
    entid: u32,
}
#[derive(Debug, Deserialize)]
pub struct GameEventIdx {
    byte: u64,
    id: i32,
}
#[derive(Debug, Deserialize)]
pub struct Stringtables {
    byte: u64,
}
pub struct GameEventBluePrint {
    byte: u64,
    entid: u32,
}

pub struct WriteCache {
    pub game_events: Vec<GameEvent>,
    pub string_tables: Vec<UserInfo>,
    pub packet_ents: Vec<PacketEntsOutput>,
    pub dt_start: u64,
    pub ge_start: u64,
}
impl WriteCache {
    pub fn new(jobresults: Vec<JobResult>, dt_start: u64, ge_start: u64) -> Self {
        let (game_events, string_tables, packet_ents) = WriteCache::filter_per_result(jobresults);
        println!("PEL {}", packet_ents.len());
        WriteCache {
            game_events: game_events,
            string_tables: string_tables,
            packet_ents: packet_ents,
            dt_start: dt_start,
            ge_start: ge_start,
        }
    }
    pub fn create_if_not_exists(&self, path: &String) {
        println!("CREATE: {}", path);
        match metadata(path) {
            Ok(md) => {}
            Err(e) => {
                create_dir(path).unwrap();
            }
        }
    }
    pub fn write_maps(&self, path: String) {
        self.create_if_not_exists(&path);

        let mut wtr = Writer::from_path(path + "maps.csv").unwrap();
        wtr.write_record(vec!["byte", "map"]).unwrap();
        wtr.write_record(vec![self.dt_start.to_string(), "dt".to_string()])
            .unwrap();
        wtr.write_record(vec![self.ge_start.to_string(), "ge".to_string()])
            .unwrap();
    }

    pub fn write_packet_ents(&mut self, path: String) {
        self.create_if_not_exists(&path);

        let mut wtr = Writer::from_path(path + "packet_ents.csv").unwrap();
        wtr.write_record(vec!["byte", "idx", "entid"]).unwrap();

        for p in &self.packet_ents {
            for x in &p.data {
                wtr.write_record(vec![
                    p.byte.to_string(),
                    x.ent_id.to_string(),
                    x.prop_inx.to_string(),
                ])
                .unwrap();
            }
        }
        wtr.flush().unwrap();
    }
    pub fn write_string_tables(&mut self, path: String) {
        self.create_if_not_exists(&path);

        let mut wtr = Writer::from_path(path + "string_tables.csv").unwrap();
        wtr.write_record(vec!["byte"]).unwrap();

        for p in &self.string_tables {
            wtr.write_record(vec![p.byte.to_string()]).unwrap();
        }
        wtr.flush().unwrap();
    }

    pub fn write_game_events(&mut self, path: String) {
        self.create_if_not_exists(&path);

        let mut wtr = Writer::from_path(path + "game_events.csv").unwrap();
        wtr.write_record(vec!["byte", "id"]).unwrap();

        for event in &self.game_events {
            wtr.write_record(vec![event.byte.to_string(), event.id.to_string()])
                .unwrap();
        }
        wtr.flush().unwrap();
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

impl ReadCache {
    pub fn new(cache_path: &String) -> Self {
        ReadCache {
            deltas: vec![],
            game_events: vec![],
            stringtables: vec![],
            cache_path: cache_path.clone(),
        }
    }

    pub fn set_deltas(&mut self) {
        let mut rdr = csv::Reader::from_path(&self.cache_path).unwrap();
        for result in rdr.deserialize() {
            let record: Delta = result.unwrap();
            self.deltas.push(record);
        }
    }
    pub fn set_game_events(&mut self) {
        let mut rdr = csv::Reader::from_path(&self.cache_path).unwrap();
        for result in rdr.deserialize() {
            let record: GameEventIdx = result.unwrap();
            if record.id == 24 {
                self.game_events.push(record);
            }
        }
    }
    pub fn set_stringtables(&mut self) {
        let mut rdr = csv::Reader::from_path(&self.cache_path).unwrap();
        for result in rdr.deserialize() {
            let record: Stringtables = result.unwrap();
            self.stringtables.push(record);
        }
    }
    pub fn get_stringtables(&self) -> Vec<u64> {
        self.stringtables.iter().map(|s| s.byte).collect()
    }
    pub fn get_event_by_id(&self, wanted_id: i32) -> Vec<&GameEventIdx> {
        self.game_events
            .iter()
            .filter(|x| x.id == wanted_id)
            .collect()
    }
    pub fn get_event_bytes_by_id(&self, wanted_id: i32) -> Vec<u64> {
        self.game_events
            .iter()
            .filter(|x| x.id == wanted_id)
            .map(|x| x.byte)
            .collect()
    }

    pub fn get_game_event_jobs(
        &mut self,
        job_results: &Vec<JobResult>,
        players: &Players,
    ) -> Vec<GameEventBluePrint> {
        let mut v = vec![];
        for event in job_results {
            match event {
                JobResult::GameEvents(ge) => {
                    if ge.len() > 0 {
                        let d = ge[0].get_key_by_name("attacker".to_string());
                        match d {
                            Some(super::game_events::KeyData::Short(s)) => {
                                let entid = players.uid_to_entid(s as u32, ge[0].byte);
                                v.push(GameEventBluePrint {
                                    byte: ge[0].byte as u64,
                                    entid: entid,
                                });
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        v
    }

    pub fn get_event_deltas(
        &self,
        wanted_id: u32,
        players: &Players,
        events: &Vec<GameEventBluePrint>,
    ) -> Vec<u64> {
        let mut v = vec![];
        //let wanted_events = self.get_event_by_id(wanted_id as i32);

        let mut kills_idx = 0;
        for i in 0..self.deltas.len() {
            let delta_start_byte = self.deltas[i].byte;

            if events[kills_idx].byte < delta_start_byte {
                let byte_want = self.find_last_val(
                    &self.deltas,
                    &events[kills_idx],
                    i,
                    wanted_id,
                    players,
                    events[kills_idx].byte,
                );
                v.push(byte_want);
                //println!("GE: {:?} {byte_want}", self.game_events);

                kills_idx += 1;
                if kills_idx == events.len() {
                    break;
                }
            }
        }
        println!("Number bytes {:?}", v.len());
        v
    }

    pub fn find_last_val(
        &self,
        v: &Vec<Delta>,
        kill: &GameEventBluePrint,
        i: usize,
        wanted_idx: u32,
        players: &Players,
        byte: u64,
    ) -> u64 {
        /*
        Find most recent delta
        */
        for i in (0..i).rev() {
            if v[i].idx == wanted_idx && v[i].entid == kill.entid {
                return v[i].byte;
            }
        }
        panic!("no found")
    }
}
