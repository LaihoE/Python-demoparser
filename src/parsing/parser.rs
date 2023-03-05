use super::cache::WriteCache;
use super::utils::IS_ARRAY_PROP;
use crate::parsing::cache::cache_reader::ReadCache;
use crate::parsing::cache::AMMO_ID;
use crate::parsing::demo_parsing::*;
use crate::parsing::parser_settings::*;
use crate::parsing::utils::CACHE_ID_MAP;
pub use crate::parsing::variants::*;
use ahash::HashMap;
#[cfg(feature = "blosc")]
use hdf5::filters::blosc_set_nthreads;
use hdf5::{File, H5Type, Result};
use itertools::Itertools;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use ndarray::{arr2, s};
use std::sync::Arc;
use std::u8;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub struct Parser {
    pub maps: Maps,
    pub settings: ParserSettings,
    pub state: ParserState,
    pub bytes: Arc<Mmap>,
}
/*
FRAME -> CMD -> NETMESSAGE----------> TYPE --> Packet entities
             -> DATATABLE                  --> Game events
             -> ...(not used)              --> Stringtables
*/

impl Parser {
    pub fn start_parsing(&mut self) {
        match ReadCache::get_cache_if_exists(&self.bytes) {
            Some(mut index) => {
                self.parse_wanted_ticks(&mut index);
            }
            None => {
                self.settings.is_cache_run = true;
                self.parse_bytes(None, false, &vec![]);
                self.write_index_file();
                match ReadCache::get_cache_if_exists(&self.bytes) {
                    Some(mut index) => {
                        self.reset_settings();
                        self.parse_wanted_ticks(&mut index);
                    }
                    None => panic!("Writing cache failed!"),
                }
            }
        }
    }
    pub fn parse_bytes(
        &mut self,
        wanted_bytes: Option<Vec<u64>>,
        should_collect: bool,
        wanted_msg: &Vec<i32>,
    ) {
        let byte_readers = ByteReader::get_byte_readers(&self.bytes, wanted_bytes);
        let n_byte_readers = byte_readers.len();

        for mut byte_reader in byte_readers {
            while byte_reader.byte_idx < byte_reader.bytes.len() as usize {
                self.state.frame_started_at = byte_reader.byte_idx as u64;
                let (cmd, tick) = byte_reader.read_frame();
                self.state.tick = tick;
                self.parse_cmd(cmd, &mut byte_reader, wanted_msg);
                if should_collect {
                    self.collect_players();
                    self.collect_weapons();
                }
                if n_byte_readers > 1 {
                    break;
                }
            }
        }
    }
    pub fn parse_cmd(&mut self, cmd: u8, byte_reader: &mut ByteReader, wanted_msg: &Vec<i32>) {
        match cmd {
            1 => self.messages_from_packet(byte_reader, wanted_msg),
            2 => self.messages_from_packet(byte_reader, wanted_msg),
            6 => self.parse_datatable(byte_reader),
            7 => { // signals end of demo
            }
            _ => {}
        }
    }
    pub fn messages_from_packet(&mut self, byte_reader: &mut ByteReader, wanted_msg: &Vec<i32>) {
        byte_reader.skip_n_bytes(160);
        let packet_len = byte_reader.read_i32();
        let goal_inx = byte_reader.byte_idx + packet_len as usize;
        while byte_reader.byte_idx < goal_inx {
            let msg = byte_reader.read_varint();
            let size = byte_reader.read_varint();
            self.msg_handler(msg, size, byte_reader, wanted_msg);
        }
    }
    pub fn msg_handler(
        &mut self,
        msg: u32,
        size: u32,
        byte_reader: &mut ByteReader,
        wanted_msg: &Vec<i32>,
    ) {
        if wanted_msg.len() != 0 && !wanted_msg.contains(&(msg as i32)) {
            byte_reader.skip_n_bytes(size);
            return;
        }
        match msg {
            12 => self.create_string_table(byte_reader, size as usize),
            13 => self.update_string_table_msg(byte_reader, size as usize),
            25 => self.parse_game_event(byte_reader, size as usize),
            26 => self.parse_packet_entities(byte_reader, size as usize),
            30 => self.parse_game_event_map(byte_reader, size as usize),
            _ => {
                byte_reader.skip_n_bytes(size);
            }
        }
    }

    pub fn write_index_file(&mut self) {
        let mut wc = WriteCache::new(&self.bytes);
        wc.write_packet_ents(&self.state.test, &self.maps.serverclass_map);
        wc.write_eid_cls_map(&self.state.eid_cls_history);
        wc.write_dt_ge_map(self.state.dt_started_at, self.state.ge_map_started_at);
        wc.write_game_events(&self.state.game_event_history);
        wc.write_stringtables(&self.state.stringtable_history);
        wc.flush();
    }

    fn parse_mandatory_ticks(&mut self, read_cache: &mut ReadCache) {
        read_cache.read_index();
        self.state.eid_cls_map = read_cache.get_eid_cls_map();

        let mut wanted_bytes = vec![];
        // 2 maps needed for parsing
        let (dt_start, ge_start) = read_cache.read_dt_ge_map();
        if dt_start == 0 && ge_start == 0 {
            println!("NO MAP");
            return;
        }
        wanted_bytes.push(dt_start);
        // Players come trough here (name, steamid, entid etc.)
        if self.settings.parse_props {
            wanted_bytes.push(dt_start);
        }
        wanted_bytes.push(ge_start);
        wanted_bytes.extend(read_cache.read_stringtables());
        self.parse_bytes(Some(wanted_bytes), false, &vec![12, 13, 30]);

        if self.settings.parse_props {
            self.maps.name_entid_prop = self.generate_name_id_map();
            self.maps.name_ptype_map = self.generate_name_ptype_map();
        }
    }

    pub fn parse_wanted_ticks(&mut self, read_cache: &mut ReadCache) {
        self.parse_mandatory_ticks(read_cache);

        let mut wanted_bytes = vec![];
        if self.settings.parse_game_events {
            let event_id = self.maps.event_name_to_id[&self.settings.event_name];
            let wanted_ticks_this_event = read_cache.get_event_ticks(event_id);
            self.settings.wanted_ticks = wanted_ticks_this_event;
            wanted_bytes.extend(read_cache.game_events_by_id(event_id));
        }

        if self.settings.parse_props {
            for prop in &self.settings.wanted_props {
                let prop_id = CACHE_ID_MAP[prop];
                let bytes = match IS_ARRAY_PROP.contains_key(prop) {
                    true => read_cache.read_by_id_others(prop_id as i32),
                    false => {
                        read_cache.read_by_id_players(prop_id as i32, &self.settings.wanted_ticks)
                    }
                };
                wanted_bytes.extend(bytes);
            }
        }
        if self.settings.wanted_props.contains(&"weapon".to_string())
            || self.settings.wanted_props.contains(&"ammo".to_string())
        {
            wanted_bytes.extend(read_cache.read_weapons());
            let creates: Vec<u64> = self
                .state
                .eid_cls_history
                .iter()
                .map(|x| x.byte as u64)
                .collect();
            wanted_bytes.extend(creates);
            wanted_bytes.sort();

            self.state.clip_id = self.maps.name_entid_prop["m_iClip1"] as i32;
            self.state.item_def_id = self.maps.name_entid_prop["m_iItemDefinitionIndex"] as i32;
            wanted_bytes.extend(read_cache.read_weapons());
            let creates: Vec<u64> = self
                .state
                .eid_cls_history
                .iter()
                .map(|x| x.byte as u64)
                .collect();
            wanted_bytes.extend(creates);
            wanted_bytes.sort();
        }

        let mut uniq: Vec<u64> = wanted_bytes.iter().map(|x| *x).unique().collect();
        uniq.sort();
        if self.settings.parse_game_events && self.settings.parse_props {
            self.parse_bytes(Some(uniq), false, &vec![]);
        } else if self.settings.parse_game_events && !self.settings.parse_props {
            self.parse_bytes(Some(uniq), false, &vec![25]);
        } else {
            self.parse_bytes(Some(uniq), true, &vec![]);
        }
    }
    fn reset_settings(&mut self) {
        self.settings.is_cache_run = false;
        self.state.output = HashMap::default();
        self.state.game_events = vec![];
        self.state.entities = HashMap::default();
    }
}
