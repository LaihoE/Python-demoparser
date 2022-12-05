use super::game_events::GameEvent;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::parse_packet_entities;
use crate::parsing::entities::Entity;
use crate::parsing::parser_settings::*;
use crate::parsing::read_bytes::ByteReader;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
use crate::parsing::utils::check_round_change;
use crate::parsing::utils::read_file;
use crate::parsing::utils::TYPEHM;
use crate::parsing::variants::BytesVariant::Mmap3;
pub use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use dashmap::DashMap;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use protobuf;
use protobuf::Message;
use rayon::prelude::*;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::collections::HashSet;
use std::slice;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;
use std::u8;
use threadpool::ThreadPool;

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
}

/*
FRAME -> CMD -> NETMESSAGE----------> TYPE --> Packet entities
             -> DATATABLE                  --> Game events
             -> ...(not used)              --> Stringtables
*/

impl Parser {
    pub fn start_parsing(&mut self, props_names: &Vec<String>) {
        let mut byte_reader = ByteReader::new(self.bytes.clone());

        while byte_reader.byte_idx < byte_reader.bytes.len() as usize {
            let (cmd, tick) = byte_reader.read_frame();
            self.state.tick = tick;
            self.parse_cmd(cmd, &mut byte_reader);
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
                self.parse_game_event_map(task);
            }
            if task.msg == 12 {
                let new_players = self.create_string_table(task);
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
        let sts: Vec<JobResult> = results.into_iter().filter(|x| x.is_stringtable()).collect();
        println!("{:?}", sts);
    }
    pub fn msg_handler(
        blueprint: &MsgBluePrint,
        bytes: &Mmap,
        serverclass_map: &HashMap<u16, ServerClass, RandomState>,
        game_events_map: &HashMap<i32, Descriptor_t, RandomState>,
        stringtables: &Vec<StringTable>,
    ) -> JobResult {
        let wanted_event = "player_blind";
        match blueprint.msg {
            26 => parse_packet_entities(blueprint, bytes, serverclass_map),
            25 => Parser::parse_game_events(blueprint, bytes, game_events_map, wanted_event),
            13 => Parser::update_string_table_msg(blueprint, bytes, stringtables),
            _ => JobResult::None,
        }
    }
}
