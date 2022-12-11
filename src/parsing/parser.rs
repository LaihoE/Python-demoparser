use super::entities::PacketEntsOutput;
use super::game_events::GameEvent;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::parse_packet_entities;
use crate::parsing::entities::Entity;
use crate::parsing::parser_settings::*;
use crate::parsing::read_bytes::ByteReader;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
pub use crate::parsing::variants::*;
use ahash::RandomState;
use core::time::Duration;
//use crossbeam_queue::ArrayQueue as SegQueue;
use crossbeam_queue::SegQueue;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use memmap2::Mmap;
use polars::export::regex::internal::Inst;
use pyo3::pyclass::boolean_struct::False;
use rayon::prelude::*;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Instant;
use std::u8;

pub struct Parser {
    pub maps: Maps,
    pub settings: ParserSettings,
    pub state: ParserState,
    pub bytes: Arc<Mmap>,
    // General purpose int vec, for perf reasons
    pub tasks: Vec<MsgBluePrint>,
    pub players: Vec<UserInfo>,
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
    PacketEntities(Option<PacketEntsOutput>),
    GameEvents(Vec<GameEvent>),
    StringTables(Vec<UserInfo>),
    DataTable(HashMap<u16, ServerClass, RandomState>),
    None,
}
#[derive(Debug)]
pub enum CmdResult {
    BluePrints(Vec<MsgBluePrint>),
    DataTable(JobResult),
}

pub struct ParsingMaps {
    pub serverclass_map: Option<HashMap<u16, ServerClass, RandomState>>,
    pub event_map: Option<HashMap<i32, Descriptor_t, RandomState>>,
}

impl Parser {
    pub fn start_parsing(&mut self, props_names: &Vec<String>) {
        let before = Instant::now();
        let q: Arc<SegQueue<Vec<MsgBluePrint>>> = Arc::new(SegQueue::new());
        println!("wat {:2?}", before.elapsed());

        let parsing_maps = Arc::new(RwLock::new(ParsingMaps {
            serverclass_map: None,
            event_map: None,
        }));

        let io_threads = 8;
        let cpu_threads = 4;

        let mut io_done = Arc::new(AtomicBool::new(false));

        let io_handles = self.start_io_threads(q.clone(), parsing_maps.clone(), io_threads);

        let parser_handles = self.start_parser_thread_main(
            q.clone(),
            parsing_maps.clone(),
            io_done.clone(),
            cpu_threads,
        );

        let before = Instant::now();
        //let mut final_data = Vec::with_capacity(500000);

        for io_handle in io_handles {
            let t = io_handle.0.join().unwrap();
        }

        io_done.store(true, Ordering::Relaxed);

        let mut cnt = 0;
        for parser_handle in parser_handles {
            let data = parser_handle.join().unwrap();
            //final_data.extend(data);
            cnt += 1;
        }

        //self.get_raw_df(&final_data, parsing_maps.clone());
    }

    pub fn start_io_threads(
        &mut self,
        q: Arc<SegQueue<Vec<MsgBluePrint>>>,
        parsing_maps: Arc<RwLock<ParsingMaps>>,
        io_threads: i32,
    ) -> Vec<(thread::JoinHandle<i32>, (usize, usize))> {
        // Referred to as IOthreads but actually do some minimal parsing too.

        // Setup byte Readers
        let starting_bytes = Parser::split_file_evenly(io_threads, self.bytes.clone());

        let mut handles = vec![];
        for start_end in starting_bytes {
            let before = Instant::now();

            let msg_q = q.clone();
            let my_maps = parsing_maps.clone();
            let my_mmap = self.bytes.clone();

            let handle =
                thread::spawn(move || Parser::parse_messages(my_mmap, msg_q, my_maps, start_end));
            handles.push((handle, start_end));
        }
        handles
    }
    pub fn start_parser_thread_main(
        &mut self,
        q: Arc<SegQueue<Vec<MsgBluePrint>>>,
        parsing_maps: Arc<RwLock<ParsingMaps>>,
        io_done: Arc<AtomicBool>,
        cpu_threads: i32,
    ) -> Vec<thread::JoinHandle<Vec<JobResult>>> {
        let mut handles = vec![];
        for thread in 0..cpu_threads {
            let mmap = self.bytes.clone();
            let msg_q = q.clone();
            let my_maps = parsing_maps.clone();
            let my_io_done = io_done.clone();
            let wanted_props = self.settings.wanted_props.clone();
            let handle = thread::spawn(move || {
                Parser::start_parser_thread(msg_q, mmap, my_maps, &wanted_props, my_io_done)
            });
            handles.push(handle);
        }
        handles
    }

    pub fn start_parser_thread(
        msg_q: Arc<SegQueue<Vec<MsgBluePrint>>>,
        mmap: Arc<Mmap>,
        parsing_maps: Arc<RwLock<ParsingMaps>>,
        wanted_props: &Vec<String>,
        io_done: Arc<AtomicBool>,
    ) -> Vec<JobResult> {
        let before = Instant::now();
        let serverclass_map = Parser::wait_for_map(parsing_maps.clone());
        let mut threads_data = Vec::with_capacity(22);
        loop {
            // println!("{}", msg_q.len());
            if msg_q.len() == 0 && io_done.load(Ordering::Relaxed) == true {
                break;
            }

            let cmd_results = msg_q.pop();
            match cmd_results {
                Some(sv) => {
                    for blueprint in sv {
                        let data = Parser::msg_handler(
                            &blueprint,
                            &mmap,
                            &serverclass_map,
                            wanted_props,
                            parsing_maps.clone(),
                        );
                        // println!("{:?}", data);
                        // threads_data.push(data);
                    }
                }
                _ => {}
            }
        }
        //println!("{}", threads_data.len());c
        return threads_data;
    }
    #[inline(always)]
    pub fn parse_messages(
        mmap: Arc<Mmap>,
        q: Arc<SegQueue<Vec<MsgBluePrint>>>,
        parsing_maps: Arc<RwLock<ParsingMaps>>,
        start_end: (usize, usize),
    ) -> i32 {
        let (start_idx, end_idx) = start_end;
        let mut byte_reader = Parser::find_beginning(mmap, start_idx, end_idx).unwrap();
        while byte_reader.byte_idx < byte_reader.max_byte as usize {
            let mut v = Vec::with_capacity(1024);
            while v.len() < 1024 && byte_reader.byte_idx < byte_reader.max_byte as usize {
                let (cmd, tick) = byte_reader.read_frame();
                let sv = Parser::parse_cmd(cmd, &mut byte_reader, tick, parsing_maps.clone());

                match sv {
                    Some(svprints) => {
                        for item in svprints {
                            v.push(item);
                        }
                    }
                    None => {}
                }
            }
            q.push(v.clone());
        }
        69
    }
    pub fn split_file_evenly(n_threads: i32, mmap: Arc<Mmap>) -> Vec<(usize, usize)> {
        let max_len = mmap.len();
        let chunk_size = (max_len / n_threads as usize) as i32;
        let mut byte_indicies = vec![];

        for i in 1..n_threads {
            let start_byte = (i * chunk_size) as usize;
            let mut end_byte = (i * chunk_size + chunk_size) as usize;
            if end_byte > max_len {
                end_byte = max_len
            }
            byte_indicies.push((start_byte, end_byte));
        }
        byte_indicies.push((1072, 150000));
        // byte_indicies.push((150000, ((chunk_size / 2) as usize)));
        // byte_indicies.push(((chunk_size / 2) as usize, (chunk_size as usize)));
        byte_indicies
    }
    #[inline(always)]
    pub fn find_beginning(mmap: Arc<Mmap>, start_idx: usize, end_idx: usize) -> Option<ByteReader> {
        if start_idx == 1072 {
            return Some(ByteReader::new(mmap.clone(), 1072, 500000));
        }
        // Jump into middle of the file and figure out where we are in the parsing
        let mut zeros_in_a_row = 0;
        let mut zeros_started_at = 0;
        let mut cur_idx = start_idx;
        for b in &mmap[start_idx..] {
            if b == &0 {
                if zeros_in_a_row == 0 {
                    zeros_started_at = cur_idx;
                }
                zeros_in_a_row += 1;
            } else {
                // 😎 no magic tricks here, move on
                if ((zeros_in_a_row == 154) && (&mmap[cur_idx - 158] == &2))
                //|| (zeros_in_a_row == 155) && (&mmap[cur_idx - 158] == &2)
                {
                    //println!("zeroloop: {} ({start_idx}, {cur_idx})", cur_idx - start_idx,);
                    if cur_idx - start_idx > 100000 {
                        println!("{}", cur_idx);
                    }
                    return Some(ByteReader::new(mmap.clone(), cur_idx - 158, end_idx));
                }
                zeros_in_a_row = 0;
            }
            cur_idx += 1;
        }
        None
    }

    #[inline(always)]
    pub fn parse_cmd(
        cmd: u8,
        byte_reader: &mut ByteReader,
        tick: i32,
        parsing_maps: Arc<RwLock<ParsingMaps>>,
    ) -> Option<SmallVec<[MsgBluePrint; 12]>> {
        match cmd {
            1 => Some(Parser::parse_packet(byte_reader, tick)),
            2 => Some(Parser::parse_packet(byte_reader, tick)),
            6 => {
                Parser::parse_datatable(byte_reader, parsing_maps.clone());
                None
            }
            _ => None,
        }
    }

    #[inline(always)]
    pub fn parse_packet(byte_reader: &mut ByteReader, tick: i32) -> SmallVec<[MsgBluePrint; 12]> {
        byte_reader.byte_idx += 160;
        let packet_len = byte_reader.read_i32();
        let packet_last_byte = byte_reader.byte_idx + packet_len as usize;
        let mut tasks = SmallVec::<[MsgBluePrint; 12]>::new();

        while byte_reader.byte_idx < packet_last_byte {
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
                tick: tick,
            };
            tasks.push(msg_blueprint)
        }
        tasks
    }

    pub fn msg_handler(
        blueprint: &MsgBluePrint,
        bytes: &Mmap,
        serverclass_map: &HashMap<u16, ServerClass, RandomState>,
        //game_events_map: &HashMap<i32, Descriptor_t, RandomState>,
        wanted_props: &Vec<String>,
        parser_maps: Arc<RwLock<ParsingMaps>>,
    ) -> JobResult {
        let wanted_event = "player_blind";
        match blueprint.msg {
            //30 => Parser::parse_game_event_map(bytes, blueprint, parser_maps),
            26 => parse_packet_entities(blueprint, bytes, serverclass_map, wanted_props),
            //25 => Parser::parse_game_events(blueprint, bytes, game_events_map, wanted_event),
            //13 => Parser::update_string_table_msg(blueprint, bytes),
            _ => JobResult::None,
        }
    }
    pub fn wait_for_map(
        parsing_maps: Arc<RwLock<ParsingMaps>>,
    ) -> HashMap<u16, ServerClass, RandomState> {
        loop {
            let parsing_maps_read = parsing_maps.read().unwrap();
            if parsing_maps_read.serverclass_map.is_some() {
                let scr = parsing_maps_read.serverclass_map.as_ref().unwrap();
                let c = scr.clone();
                drop(scr);
                return c;
            } else {
                let ten_millis = Duration::from_micros(10);
                thread::sleep(ten_millis);
            };
        }
    }
}
