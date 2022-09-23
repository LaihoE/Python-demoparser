mod parsing;
use csgoproto::netmessages;
use csgoproto::steammessages;
use hashbrown::HashMap;
use jemallocator;
use parsing::header::Header;
use parsing::parser::Demo;
use protobuf::reflect::MessageDescriptor;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use std::convert::TryInto;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    //demo_name = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571866312135147584_0815469279_189.dem"
    //let demo_path = "/home/laiho/.steam/steam/steamapps/common/Counter-Strike Global Offensive/csgo/replays/match730_003571109800890597417_2128991285_181.dem".to_string();

    //let demo_path = "/home/laiho/Documents/demos/rclonetest/w.dem";
    let demo_path =
        "/home/laiho/Documents/demos/rclonetest/match730_003449965367076585902_0881240613_184.dem";
    let props_names = vec!["m_vecOrigin_X".to_string()];
    let x = netmessages::file_descriptor();
    let y = x.messages();
    let mut v: Vec<MessageDescriptor> = Vec::new();
    let mut cnt = 0;

    let mut d = Demo {
        bytes: std::fs::read(demo_path).unwrap(),
        fp: 0,
        cmd: 0,
        tick: 0,
        event_list: None,
        event_map: None,
        dt_map: Arc::new(Mutex::new(Some(HashMap::new()))),
        class_bits: 0,
        serverclass_map: Arc::new(Mutex::new(HashMap::new())),
        entities: Arc::new(Mutex::new(Some(HashMap::new()))),
        bad: Vec::new(),
        stringtables: Vec::new(),
        players: Vec::new(),
        parse_props: true,
        game_events: Vec::new(),
        event_name: "".to_string(),
        wanted_props: Vec::new(),
        cnt: 0,
        handles: Vec::new(),
        threads_spawned: 0,
        closed_handles: 0,
        pool: rayon::ThreadPoolBuilder::new()
            .num_threads(20)
            .build()
            .unwrap(),
    };

    let h: Header = d.parse_header();
    let mut event_names: Vec<String> = Vec::new();
    use std::time::Instant;
    let now = Instant::now();
    let data = d.parse_frame(&props_names);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    println!("{}", d.cnt);
    for player in &d.players {
        println!("{} {}", player.entity_id, player.name)
    }
    println!("{:?}", &d.players.len());
}
