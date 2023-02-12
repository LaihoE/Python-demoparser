use crate::parsing::demo_parsing::ServerClass;
use ahash::HashMap;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use itertools::izip;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{fs, time::Instant};
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Delta {
    pub byte: u64,
    pub entid: u32,
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
pub struct GameEventBluePrint {
    pub byte: u64,
    pub entid: u32,
}

pub struct ReadCache {
    pub deltas: HashMap<String, Vec<Delta>>,
    pub game_events: Vec<GameEventIdx>,
    pub stringtables: Vec<Stringtables>,
    pub cache_path: String,
    pub zip: ZipArchive<File>,
}
const HASH_BYTE_LENGTH: usize = 10000;

impl ReadCache {
    pub fn new(cache_path: &String) -> Self {
        let file = fs::File::open(cache_path.to_owned()).unwrap();

        ReadCache {
            zip: ZipArchive::new(file).unwrap(),
            deltas: HashMap::default(),
            game_events: vec![],
            stringtables: vec![],
            cache_path: cache_path.clone(),
        }
    }

    pub fn get_cache_path(bytes: &[u8]) -> String {
        let file_hash = sha256::digest(&bytes[..HASH_BYTE_LENGTH]);
        let path = "/home/laiho/Documents/cache/".to_owned();
        path + &file_hash + &".zip"
    }

    pub fn get_cache_if_exists(bytes: &[u8]) -> Option<ReadCache> {
        let cache_path = Self::get_cache_path(&bytes);
        match Path::new(&cache_path).exists() {
            true => Some(ReadCache::new(&cache_path)),
            false => None,
        }
    }
    fn find_id_from_map(
        &self,
        name: &String,
        event_map: &Option<HashMap<i32, Descriptor_t>>,
    ) -> Option<i32> {
        let map = event_map.as_ref().unwrap();
        for (k, v) in map {
            if v.name() == name {
                return Some(*k);
            }
        }
        None
    }

    pub fn event_bytes_by_name(
        &self,
        name: String,
        event_map: &Option<HashMap<i32, Descriptor_t>>,
    ) -> Vec<u64> {
        let wanted_id = match self.find_id_from_map(&name, event_map) {
            Some(id) => id,
            None => panic!("No id found for game event: {}", name),
        };
        self.game_events
            .iter()
            .filter(|x| x.id == wanted_id)
            .map(|x| x.byte)
            .collect()
    }

    pub fn get_player_messages(&mut self) -> Vec<u64> {
        self.read_stringtables();
        // self.read_game_events();

        let (ge_start, dt_start) = self.read_maps();
        let mut wanted_bytes = vec![];
        wanted_bytes.push(ge_start as u64);
        wanted_bytes.push(dt_start as u64);

        wanted_bytes.extend(self.get_stringtables());
        wanted_bytes
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
    pub fn read_other_deltas_by_name(
        &mut self,
        wanted_name: &str,
        sv_cls_map: &HashMap<u16, ServerClass>,
        cls_id: u16,
    ) {
        let mut data = vec![];
        match self.zip.by_name(&wanted_name) {
            Ok(mut zip) => {
                zip.read_to_end(&mut data).unwrap();
            }
            Err(e) => {
                println!("not found {}", wanted_name);
                return;
            }
        }

        // First 8 bytes = number of "rows"
        let number_structs = usize::from_le_bytes(data[..8].try_into().unwrap());

        let mut starting_bytes = Vec::with_capacity(number_structs);
        let mut entids = Vec::with_capacity(number_structs);
        let mut ticks = Vec::with_capacity(number_structs);
        // Stored as u64
        let BYTES_SIZE = 8;
        // Stored as u32
        let PIDX_SIZE = 4;

        let bytes_starts_at = 8;
        let bytes_end_at = number_structs * BYTES_SIZE + 8;
        let entids_start_at = bytes_end_at;
        let entids_end_at = bytes_end_at + PIDX_SIZE * number_structs;
        let ticks_start_at = entids_end_at;

        for bytes in data[bytes_starts_at..bytes_end_at].chunks(BYTES_SIZE) {
            starting_bytes.push(usize::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[ticks_start_at..].chunks(PIDX_SIZE) {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[entids_start_at..entids_end_at].chunks(PIDX_SIZE) {
            entids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }

        assert_eq!(number_structs, starting_bytes.len());
        assert_eq!(number_structs, entids.len());
        assert_eq!(number_structs, ticks.len());

        let p: Vec<&str> = wanted_name.split("@").collect();
        let dot: Vec<&str> = wanted_name.split(".").collect();
        let table_name_temp: Vec<&str> = p[1].split(".").collect();
        let table_name = table_name_temp[0];
        let prop_n = dot[dot.len() - 1];
        let prefix = p[0];

        let p = &sv_cls_map[&cls_id];
        for prop in &p.props {
            if &prop.table == table_name && &prop.name == prop_n {
                let key = prefix.to_owned() + "@" + &prop.table + "." + &prop.name;
                self.deltas.insert(key, vec![]);
            }
        }
        self.deltas.insert("m_vecOrigin_X".to_owned(), vec![]);
        self.deltas.insert("m_vecOrigin_Y".to_owned(), vec![]);
        println!("WANTED {}", wanted_name);
        let v = match self.deltas.get_mut(wanted_name) {
            Some(dv) => dv,
            None => return,
        };
        v.reserve(number_structs);

        for (byte, entid, tick) in izip!(&starting_bytes, &ticks, &entids) {
            // println!("{} {} {}", byte, entid, tick);
            v.push(Delta {
                byte: *byte as u64,
                entid: *entid as u32,
                tick: *tick,
            });
        }
    }

    pub fn read_deltas_by_name(
        &mut self,
        wanted_name_temp: &str,
        sv_cls_map: &HashMap<u16, ServerClass>,
    ) {
        /*
        File format:
        first 8 bytes -> number of structs (u64)
        byte * number of structs (u64)
        pidx * number of structs (u16)

        We are storing the structs in SOA form.
        */

        let wanted_name = if wanted_name_temp.contains("m_vecOrigin") {
            "player@DT_CSNonLocalPlayerExclusive.m_vecOrigin"
        } else {
            wanted_name_temp
        };

        let mut data = vec![];
        match self.zip.by_name(&wanted_name) {
            Ok(mut zip) => {
                zip.read_to_end(&mut data).unwrap();
            }
            Err(e) => {
                println!("no found file {}", wanted_name);
                return;
            }
        }

        // First 8 bytes = number of "rows"
        let number_structs = usize::from_le_bytes(data[..8].try_into().unwrap());

        let mut starting_bytes = Vec::with_capacity(number_structs);
        let mut entids = Vec::with_capacity(number_structs);
        let mut ticks = Vec::with_capacity(number_structs);
        // Stored as u64
        let BYTES_SIZE = 8;
        // Stored as u32
        let PIDX_SIZE = 4;

        let bytes_starts_at = 8;
        let bytes_end_at = number_structs * BYTES_SIZE + 8;
        let entids_start_at = bytes_end_at;
        let entids_end_at = bytes_end_at + PIDX_SIZE * number_structs;
        let ticks_start_at = entids_end_at;

        for bytes in data[bytes_starts_at..bytes_end_at].chunks(BYTES_SIZE) {
            starting_bytes.push(usize::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[ticks_start_at..].chunks(PIDX_SIZE) {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[entids_start_at..entids_end_at].chunks(PIDX_SIZE) {
            entids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }

        assert_eq!(number_structs, starting_bytes.len());
        assert_eq!(number_structs, entids.len());
        assert_eq!(number_structs, ticks.len());

        let p: Vec<&str> = wanted_name.split("@").collect();
        let dot: Vec<&str> = wanted_name.split(".").collect();
        let table_name_temp: Vec<&str> = p[1].split(".").collect();
        let table_name = table_name_temp[0];
        let prop_n = dot[dot.len() - 1];
        let prefix = p[0];

        let p = &sv_cls_map[&40];
        for prop in &p.props {
            if &prop.table == table_name && &prop.name == prop_n {
                let key = prefix.to_owned() + "@" + &prop.table + "." + &prop.name;
                self.deltas.insert(key, vec![]);
            }
        }

        self.deltas.insert(
            "player@DT_CSNonLocalPlayerExclusive.m_vecOrigin".to_owned(),
            vec![],
        );

        let v = self.deltas.get_mut(wanted_name).unwrap();
        v.reserve(number_structs);

        for (byte, entid, tick) in izip!(&starting_bytes, &ticks, &entids) {
            // println!("{} {} {}", byte, entid, tick);
            v.push(Delta {
                byte: *byte as u64,
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
}
