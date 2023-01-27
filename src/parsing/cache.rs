use ahash::HashMap;
use csv::Writer;
use rayon::prelude::IntoParallelIterator;
use std::fs::{create_dir, metadata};
use std::{fs, time::Instant};

use super::{
    entities::PacketEntsOutput,
    game_events::{self, GameEvent},
    parser::JobResult,
    players::Players,
    stringtables::UserInfo,
};
use crate::parsing::parser;
use itertools::Itertools;
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_cbor;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Delta {
    byte: u64,
    idx: u32,
    entid: u32,
    tick: i32,
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
pub struct GameEventBluePrint {
    pub byte: u64,
    pub entid: u32,
}

pub struct WriteCache {
    pub game_events: Vec<GameEvent>,
    pub string_tables: Vec<UserInfo>,
    pub packet_ents: Vec<PacketEntsOutput>,
    pub dt_start: u64,
    pub ge_start: u64,
    pub zip: ZipWriter<File>,
}

pub struct ReadCache {
    pub deltas: Vec<Delta>,
    pub game_events: Vec<GameEventIdx>,
    pub stringtables: Vec<Stringtables>,
    pub cache_path: String,
    pub zip: ZipArchive<File>,
}
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
    pub fn write_all_caches(&mut self) {
        self.write_packet_ents();
        self.write_game_events();
        self.write_string_tables();
        self.write_maps();
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
    pub fn write_maps(&mut self) {
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        self.zip.start_file("maps", options).unwrap();

        let mut byt = vec![];
        byt.extend(self.ge_start.to_le_bytes());
        byt.extend(self.dt_start.to_le_bytes());

        self.zip.write_all(&byt).unwrap();
    }

    pub fn write_packet_ents(&mut self) {
        let forbidden = vec![0, 1, 2, 37, 103, 93, 59, 58, 1343, 1297, 40, 41, 26, 27];

        let mut v = vec![];
        for p in &self.packet_ents {
            for x in &p.data {
                if !forbidden.contains(&x.prop_inx) || x.ent_id > 64 {
                    v.push((p.byte, x.prop_inx, x.ent_id, p.tick))
                }
            }
        }

        let m = v.iter().into_group_map_by(|x| x.1);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Zstd);

        /*
        Write data per prop. Also use SoA form for ~3x size reduction.
        */

        for i in 0..2000 {
            match m.get(&i) {
                Some(g) => {
                    self.zip.start_file(i.to_string(), options).unwrap();
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

impl ReadCache {
    pub fn new(cache_path: &String) -> Self {
        let file = fs::File::open(cache_path.to_owned()).unwrap();

        ReadCache {
            zip: ZipArchive::new(file).unwrap(),
            deltas: vec![],
            game_events: vec![],
            stringtables: vec![],
            cache_path: cache_path.clone(),
        }
    }
    pub fn read_maps(&mut self) -> (usize, usize) {
        let mut data = vec![];
        let x = self
            .zip
            .by_name("maps")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        // First 8 bytes = pos where game events map starts
        // Next 8 bytes = pos where dt map starts
        let ge_map = usize::from_le_bytes(data[..8].try_into().unwrap());
        let dt_map = usize::from_le_bytes(data[8..16].try_into().unwrap());
        return (ge_map, dt_map);
    }

    pub fn read_deltas_by_pidx(&mut self, wanted_idx: u32) {
        /*
        File format:
        first 8 bytes -> number of structs (u64)
        byte * number of structs (u64)
        pidx * number of structs (u16)

        We are storing the structs in SOA form.
        */
        let mut data = vec![];
        let x = self
            .zip
            .by_name(&wanted_idx.to_string())
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        // First 8 bytes = number of "rows"
        let number_rows = usize::from_le_bytes(data[..8].try_into().unwrap());

        let mut starting_bytes = Vec::with_capacity(number_rows);
        let mut entids = Vec::with_capacity(number_rows);
        let mut ticks = Vec::with_capacity(number_rows);
        // Stored as u64
        let BYTES_SIZE = 8;
        // Stored as u32
        let PIDX_SIZE = 4;

        for bytes in data[8..number_rows * BYTES_SIZE + 8].chunks(BYTES_SIZE) {
            starting_bytes.push(usize::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[number_rows * BYTES_SIZE + 8
            ..(number_rows * BYTES_SIZE + 8) + number_rows * PIDX_SIZE + 8]
            .chunks(PIDX_SIZE)
        {
            entids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in
            data[(number_rows * BYTES_SIZE + 8) + number_rows * PIDX_SIZE + 8..].chunks(PIDX_SIZE)
        {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        use itertools::izip;

        for (byte, entid, tick) in izip!(&starting_bytes, &ticks, &entids) {
            self.deltas.push(Delta {
                byte: *byte as u64,
                idx: wanted_idx,
                entid: *entid as u32,
                tick: *tick,
            });
        }
    }
    pub fn read_game_events(&mut self) {
        let mut data = vec![];
        self.zip
            .by_name("game_events")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        // First 8 bytes = number of "rows"
        let number_rows = usize::from_le_bytes(data[..8].try_into().unwrap());

        let mut starting_bytes = Vec::with_capacity(number_rows);
        let mut ids = Vec::with_capacity(number_rows);
        // Stored as u64
        const BYTES_SIZE: usize = 8;
        // Stored as u32
        const IDS_SIZE: usize = 4;

        for bytes in data[8..number_rows * BYTES_SIZE + 8].chunks(BYTES_SIZE) {
            starting_bytes.push(usize::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[number_rows * BYTES_SIZE + 8..].chunks(IDS_SIZE) {
            ids.push(u32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for (byte, id) in starting_bytes.iter().zip(ids) {
            self.game_events.push(GameEventIdx {
                byte: *byte as u64,
                id: id as i32,
            });
        }
    }
    pub fn read_stringtables(&mut self) {
        let mut data = vec![];
        self.zip
            .by_name("string_tables")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        // First 8 bytes = number of "rows"
        let number_rows = usize::from_le_bytes(data[..8].try_into().unwrap());

        let mut starting_bytes = Vec::with_capacity(number_rows);
        // Stored as u64
        const BYTES_SIZE: usize = 8;

        for bytes in data[..number_rows * BYTES_SIZE + 8].chunks(BYTES_SIZE) {
            starting_bytes.push(usize::from_le_bytes(bytes.try_into().unwrap()));
        }

        for byte in starting_bytes {
            self.stringtables.push(Stringtables { byte: byte as u64 });
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
                                match players.uid_to_entid(s as u32, ge[0].byte) {
                                    Some(entid) => {
                                        v.push(GameEventBluePrint {
                                            byte: ge[0].byte as u64,
                                            entid: entid,
                                        });
                                    }
                                    None => {}
                                }
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
        &mut self,
        wanted_id: u32,
        events: &mut Vec<GameEventBluePrint>,
    ) -> Vec<u64> {
        self.read_deltas_by_pidx(wanted_id);
        let mut v = vec![];
        let mut kills_idx = 0;

        events.sort_by_key(|x| x.byte);

        for event in events {
            let x = self.find_delta_tick(&self.deltas, event.entid, 21, event.byte);
            match x {
                Some(d) => v.push(d),
                None => {}
            }
        }
        v
    }
    pub fn find_delta_ticks(&mut self, entid: u32, prop: u32) -> Vec<u64> {
        self.read_deltas_by_pidx(prop);
        let before = Instant::now();

        let v: Vec<u64> = self
            .deltas
            .iter()
            .filter(|x| x.idx == prop && x.entid == entid)
            .map(|x| x.byte)
            .collect();

        println!("CHANGED: {:?}", v);
        println!("{:2?}", before.elapsed());
        v
    }

    #[inline]
    pub fn find_delta_tick(
        &self,
        deltas: &Vec<Delta>,
        entid: u32,
        prop: u32,
        byte: u64,
    ) -> Option<u64> {
        let startpos =
            deltas.binary_search_by(|segment| segment.byte.partial_cmp(&byte).expect("NaN"));

        let startpos = match startpos {
            Err(e) => e,
            Ok(x) => x,
        };

        for i in (0..startpos).rev() {
            let current_delta = &deltas[i];
            if current_delta.entid == entid && current_delta.idx == prop {
                // println!(">> {} {}", current_delta.tick, &deltas[startpos].tick);
                if current_delta.tick != deltas[startpos - 1].tick {
                    return Some(current_delta.byte);
                } else {
                    continue;
                }
            }
        }
        None
    }
}
