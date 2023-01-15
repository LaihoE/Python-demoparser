use super::cache::Cache;
use super::game_events::GameEvent;
use crate::parsing::cache;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::parse_packet_entities;
use crate::parsing::parser_settings::*;
use crate::parsing::players::Players;
use crate::parsing::read_bytes::ByteReader;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
pub use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use memmap2::Mmap;
use rayon::prelude::*;

use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;
use std::u8;

pub struct Parser {
    pub maps: Maps,
    pub settings: ParserSettings,
    pub state: ParserState,
    pub bytes: Arc<Mmap>,
    // General purpose int vec, for perf reasons
    pub tasks: Vec<MsgBluePrint>,
}
#[derive(Debug, Clone)]
pub struct MsgBluePrint {
    pub msg: u32,
    pub start_idx: usize,
    pub end_idx: usize,
    pub tick: i32,
    pub byte: usize,
}
#[derive(Debug)]
pub enum JobResult {
    PacketEntities(Option<Vec<SmallVec<[(i32, PropData); 1]>>>),
    GameEvents(Vec<GameEvent>),
    StringTables(Vec<UserInfo>),
    None,
}
impl JobResult {
    pub fn is_stringtable(&self) -> bool {
        match self {
            JobResult::StringTables(_) => true,
            _ => false,
        }
    }
    pub fn is_game_event(&self) -> bool {
        match self {
            JobResult::GameEvents(_) => true,
            _ => false,
        }
    }
}

/*
FRAME -> CMD -> NETMESSAGE----------> TYPE --> Packet entities
             -> DATATABLE                  --> Game events
             -> ...(not used)              --> Stringtables
*/

impl Parser {
    fn get_byte_readers(&mut self, start_pos: Vec<u64>) -> Vec<ByteReader> {
        if start_pos.len() == 0 {
            return vec![ByteReader::new(self.bytes.clone(), false, 1072)];
        }
        let mut readers = vec![];
        let data_table_reader = ByteReader::new(self.bytes.clone(), true, 138837 - 6);
        readers.push(data_table_reader);
        let data_table_reader = ByteReader::new(self.bytes.clone(), true, 458399);
        readers.push(data_table_reader);
        for pos in start_pos {
            let other_reader = ByteReader::new(self.bytes.clone(), true, pos as usize);
            readers.push(other_reader);
        }
        return readers;
    }
    pub fn start_parsing(&mut self, props_names: &Vec<String>) {
        let mut c = Cache {
            deltas: vec![],
            game_events: vec![],
            stringtables: vec![],
        };
        // 24 player death
        // GL: 458399   MAP: 138837
        c.set_deltas();
        c.set_game_events();
        c.set_stringtables();

        println!("ST {:?}", c.stringtables);
        let mut deaths = c.get_event_by_id(24);
        let st = c.get_stringtables();
        deaths.extend(st);

        let byte_readers = self.get_byte_readers(deaths);
        for mut byte_reader in byte_readers {
            let mut frames_parsed = 0;
            while byte_reader.byte_idx < byte_reader.bytes.len() as usize {
                if byte_reader.single && frames_parsed > 0 {
                    break;
                }
                let (cmd, tick) = byte_reader.read_frame();
                self.state.tick = tick;
                self.parse_cmd(cmd, &mut byte_reader);
                frames_parsed += 1;
            }
        }
        self.compute_jobs();
    }

    #[inline(always)]
    pub fn parse_cmd(&mut self, cmd: u8, byte_reader: &mut ByteReader) {
        match cmd {
            1 => self.parse_packet(byte_reader),
            2 => self.parse_packet(byte_reader),
            6 => self.parse_datatable(byte_reader),
            _ => {}
        }
    }

    #[inline(always)]
    pub fn parse_packet(&mut self, byte_reader: &mut ByteReader) {
        byte_reader.byte_idx += 160;
        let packet_len = byte_reader.read_i32();
        let goal_inx = byte_reader.byte_idx + packet_len as usize;

        while byte_reader.byte_idx < goal_inx {
            let msg = byte_reader.read_varint();
            let size = byte_reader.read_varint();
            // Get byte boundaries for this msg
            let before_inx = byte_reader.byte_idx.clone();
            byte_reader.byte_idx += size as usize;
            let after_inx = byte_reader.byte_idx.clone();
            // Information needed to parse a msg, passed to threads as a "job"
            let msg_blueprint = MsgBluePrint {
                msg: msg,
                start_idx: before_inx,
                end_idx: after_inx,
                tick: self.state.tick,
                byte: byte_reader.byte_idx - 166,
            };
            self.tasks.push(msg_blueprint);
        }
    }
    pub fn compute_jobs(&mut self) {
        let tasks = self.tasks.clone();
        // Special msg that is needed for parsing game events.
        // Event comes one time per demo sometime in the beginning
        for task in &tasks {
            if task.msg == 30 {
                println!("HERE");
                self.parse_game_event_map(task);
            }
        }
        let results: Vec<JobResult> = tasks
            .into_par_iter()
            .map(|t| {
                Parser::msg_handler(
                    &t,
                    &self.bytes,
                    &self.maps.serverclass_map,
                    &self.maps.event_map.as_ref().unwrap(),
                    &self.state.stringtables,
                )
            })
            .collect();
        //println!("{:?}", results);
        let jr: Vec<JobResult> = results.into_iter().filter(|x| x.is_stringtable()).collect();
        let p = Players::new(&jr);
        //println!("{:?}", sts);

        for x in jr {
            match x {
                JobResult::GameEvents(g) => {
                    if g.len() > 0 {
                        let d = g[0].get_key_by_name("attacker".to_string());
                        match d {
                            Some(super::game_events::KeyData::Short(s)) => {
                                p.uid_to_entid(s, g[0].tick);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    pub fn msg_handler(
        blueprint: &MsgBluePrint,
        bytes: &Mmap,
        serverclass_map: &HashMap<u16, ServerClass, RandomState>,
        game_events_map: &HashMap<i32, Descriptor_t, RandomState>,
        stringtables: &Vec<StringTable>,
    ) -> JobResult {
        let wanted_event = "player_death";
        match blueprint.msg {
            26 => parse_packet_entities(blueprint, bytes, serverclass_map),
            25 => Parser::parse_game_events(blueprint, bytes, game_events_map, wanted_event),
            12 => Parser::create_string_table(blueprint, bytes),
            13 => Parser::update_string_table_msg(blueprint, bytes),
            _ => JobResult::None,
        }
    }
}
