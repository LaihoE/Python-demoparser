/*
use crate::parsing::cache::PLAYER_CLSID;
use crate::parsing::demo_parsing::entities_cache_only::EidClsHistoryEntry;
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
    pub game_events: Vec<GameEventIdx>,
    pub stringtables: Vec<Stringtables>,
    pub cache_path: String,
    pub zip: ZipArchive<File>,
    pub idx_byte_map: Vec<u64>,
    pub idx_tick_map: Vec<i32>,
    pub eid_cls_map: Vec<EidClsHistoryEntry>,
}
const HASH_BYTE_LENGTH: usize = 10000;

impl ReadCache {
    pub fn new(cache_path: &String) -> Self {
        let file = fs::File::open(cache_path.to_owned()).unwrap();
        let before = Instant::now();

        let mut ziparc = ZipArchive::new(file).unwrap();
        let (idx_byte_map, idx_tick_map) = ReadCache::read_byte_tick_map(&mut ziparc);

        let rc = ReadCache {
            zip: ziparc,
            deltas: HashMap::default(),
            game_events: vec![],
            stringtables: vec![],
            cache_path: cache_path.clone(),
            idx_byte_map: idx_byte_map,
            idx_tick_map: idx_tick_map,
            eid_cls_map: vec![],
        };

        println!("{:2?}", before.elapsed());
        rc
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
        let mut wanted_bytes = vec![];

        self.read_stringtables();
        let (ge_start, dt_start) = self.read_maps();
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
                return;
            }
        }
        // First 8 bytes = number of "rows"
        let number_structs = usize::from_le_bytes(data[..8].try_into().unwrap());
        let mut idx = Vec::with_capacity(number_structs);
        let mut entids = Vec::with_capacity(number_structs);
        // Stored as i32
        let BYTES_SIZE = 4;
        // Stored as i32
        let PIDX_SIZE = 2;

        let bytes_starts_at = 8;
        let bytes_end_at = number_structs * BYTES_SIZE + 8;
        let entids_start_at = bytes_end_at;

        for bytes in data[bytes_starts_at..bytes_end_at].chunks(BYTES_SIZE) {
            idx.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[entids_start_at..].chunks(PIDX_SIZE) {
            entids.push(i16::from_le_bytes(bytes.try_into().unwrap()));
        }

        assert_eq!(number_structs, entids.len());
        assert_eq!(number_structs, idx.len());
        assert_eq!(entids.len(), idx.len());

        let mut starting_bytes = vec![];
        let mut ticks = vec![];
        let mut entids_out = vec![];

        for (idx, entid) in idx.iter().zip(entids) {
            starting_bytes.push(self.idx_byte_map[*idx as usize]);
            ticks.push(self.idx_tick_map[*idx as usize]);
            entids_out.push(entid);
        }

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

        self.deltas.insert(
            "player@DT_CSNonLocalPlayerExclusive.m_vecOrigin".to_owned(),
            vec![],
        );

        let v = self.deltas.get_mut(wanted_name).unwrap();
        v.reserve(number_structs);

        for (byte, entid, tick) in izip!(&starting_bytes, &entids_out, &ticks) {
            v.push(Delta {
                byte: *byte as u64,
                entid: *entid,
                tick: *tick,
            });
        }
    }
    fn read_zip_to_buffer(&mut self, name: &str) -> Result<Vec<u8>, ZipError> {
        let mut bytes = vec![];
        let mut zf = self.zip.by_name(name)?;
        zf.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub fn read_byte_tick_map(zip: &mut ZipArchive<File>) -> (Vec<u64>, Vec<i32>) {
        let mut data = vec![];
        let mut zf = zip.by_name("tick_mapping").unwrap();
        zf.read_to_end(&mut data).unwrap();

        let number_structs = usize::from_le_bytes(data[..8].try_into().unwrap());
        let mut idx = Vec::with_capacity(number_structs);
        let mut start_bytes = Vec::with_capacity(number_structs);
        let mut ticks = Vec::with_capacity(number_structs);
        // Stored as u64
        let IDX_SIZE = 4;
        let BYTE_SIZE = 8;
        let TICK_SIZE = 4;

        let idx_start_at = 8;
        let idx_end_at = number_structs * IDX_SIZE + 8;
        let bytes_start_at = idx_end_at;
        let bytes_end_at = idx_end_at + number_structs * BYTE_SIZE;
        let ticks_start_at = bytes_end_at;

        for bytes in data[idx_start_at..idx_end_at].chunks(IDX_SIZE) {
            idx.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[bytes_start_at..bytes_end_at].chunks(BYTE_SIZE) {
            start_bytes.push(u64::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[ticks_start_at..].chunks(TICK_SIZE) {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        // Missing "value". Cant really use same missing value due to ranges :(
        let mut idx_byte_map = vec![9999999999; idx.len()];
        let mut idx_tick_map = vec![-9999999; idx.len()];

        let before = Instant::now();

        for (i, byte, tick) in izip!(&idx, &start_bytes, &ticks) {
            idx_byte_map[*i as usize] = *byte;
            idx_tick_map[*i as usize] = *tick;
        }
        println!("HM TOOK: {:2?}", before.elapsed());

        (idx_byte_map, idx_tick_map)
    }
    fn extract_entid_mask(
        &mut self,
        indicies: Vec<i32>,
        entids: Vec<i16>,
    ) -> (Vec<u64>, Vec<i32>, Vec<i16>) {
        let mut starting_bytes = vec![];
        let mut ticks = vec![];
        let mut entids_out = vec![];
        println!("{}", entids.len());

        for (idx, entid) in indicies.iter().zip(entids) {
            starting_bytes.push(self.idx_byte_map[*idx as usize]);
            ticks.push(self.idx_tick_map[*idx as usize]);
            entids_out.push(entid);
        }

        (starting_bytes, ticks, entids_out)
    }

    pub fn read_deltas_by_name(
        &mut self,
        wanted_name_temp: &str,
        sv_cls_map: &HashMap<u16, ServerClass>,
    ) {
        let wanted_name = if wanted_name_temp.contains("m_vecOrigin") {
            "player@DT_CSNonLocalPlayerExclusive.m_vecOrigin"
        } else {
            wanted_name_temp
        };
        let data = match self.read_zip_to_buffer(wanted_name) {
            Ok(data) => data,
            Err(e) => panic!("no tick map found! {}", e),
        };
        // IF this fails then GG
        // First 8 bytes = number of "rows"
        let number_structs = usize::from_le_bytes(data[..8].try_into().unwrap());

        let mut indicies = Vec::with_capacity(number_structs);
        let mut entids = Vec::with_capacity(number_structs);
        // Stored as i32
        let BYTES_SIZE = 4;
        let PIDX_SIZE = 2;

        let bytes_starts_at = 8;
        let bytes_end_at = number_structs * BYTES_SIZE + 8;
        let entids_start_at = bytes_end_at;

        for bytes in data[bytes_starts_at..bytes_end_at].chunks(BYTES_SIZE) {
            indicies.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[entids_start_at..].chunks(PIDX_SIZE) {
            entids.push(i16::from_le_bytes(bytes.try_into().unwrap()));
        }
        assert_eq!(number_structs, entids.len());
        assert_eq!(number_structs, indicies.len());
        //assert_eq!(entids.len(), indicies.len());

        let (starting_bytes, ticks, entids_out) = self.extract_entid_mask(indicies, entids);

        self.init_delta_hm(wanted_name, sv_cls_map);

        let v = self.deltas.get_mut(wanted_name).unwrap();
        v.reserve(number_structs);

        for (byte, entid, tick) in izip!(&starting_bytes, &entids_out, &ticks) {
            //println!("{} {} {}", byte, entid, tick);
            v.push(Delta {
                byte: *byte as u64,
                entid: *entid,
                tick: *tick,
            });
        }
    }
    fn init_delta_hm(&mut self, wanted_name: &str, sv_cls_map: &HashMap<u16, ServerClass>) {
        let p: Vec<&str> = wanted_name.split("@").collect();
        let dot: Vec<&str> = wanted_name.split(".").collect();
        let table_name_temp: Vec<&str> = p[1].split(".").collect();
        let table_name = table_name_temp[0];
        let prop_n = dot[dot.len() - 1];
        let prefix = p[0];

        let p = &sv_cls_map[&PLAYER_CLSID];
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
        const BYTES_SIZE: usize = 8;

        for bytes in data[..number_rows * BYTES_SIZE + 8].chunks(BYTES_SIZE) {
            starting_bytes.push(usize::from_le_bytes(bytes.try_into().unwrap()));
        }

        for byte in starting_bytes {
            self.stringtables.push(Stringtables { byte: byte as u64 });
        }
    }

    pub fn read_eid_cls_map(&mut self) {
        let mut data = vec![];
        self.zip
            .by_name("eid_cls_map")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        // First 8 bytes = number of "rows"
        let number_structs = usize::from_le_bytes(data[..8].try_into().unwrap());

        let mut eids = Vec::with_capacity(number_structs);
        let mut cls_ids = Vec::with_capacity(number_structs);
        let mut ticks = Vec::with_capacity(number_structs);

        const EIDS_SIZE: usize = 4;
        const CLS_SIZE: usize = 2;
        const TICKS_SIZE: usize = 4;

        let eids_start_at = 8;
        let eids_end_at = number_structs * EIDS_SIZE + 8;
        let cls_start_at = eids_end_at;
        let cls_end_at = cls_start_at + number_structs * CLS_SIZE;

        let ticks_start_at = cls_end_at;

        println!("NS {}", number_structs);

        for bytes in data[eids_start_at..eids_end_at].chunks(EIDS_SIZE) {
            eids.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[cls_start_at..cls_end_at].chunks(CLS_SIZE) {
            cls_ids.push(u16::from_le_bytes(bytes.try_into().unwrap()));
        }
        for bytes in data[ticks_start_at..].chunks(TICKS_SIZE) {
            ticks.push(i32::from_le_bytes(bytes.try_into().unwrap()));
        }
        println!("{} {} {}", eids.len(), cls_ids.len(), ticks.len());

        for (eid, cls_id, tick) in izip!(eids, cls_ids, ticks) {
            self.eid_cls_map.push(EidClsHistoryEntry {
                eid: eid,
                cls_id: cls_id,
                tick: tick,
            });
            //println!("{} {} {}", eid, cls_id, tick);
        }
    }
}
*/
