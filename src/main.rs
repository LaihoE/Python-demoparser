mod parsing;
use hashbrown::HashMap;
use jemallocator;
use parsing::header::Header;
use parsing::parser::Demo;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use std::convert::TryInto;
use std::time::Instant;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let demo_path = "/home/laiho/Documents/demos/rclonetest/w.dem";
    //let demo_path = "/home/laiho/Documents/demos/rclonetest/xx.dem";
    let props_names = vec!["m_vecOrigin_X".to_string()];

    let mut d = Demo {
        bytes: std::fs::read(demo_path).unwrap(),
        fp: 0,
        cmd: 0,
        tick: 0,
        event_list: None,
        event_vec: None,
        dt_map: Some(HashMap::default()),
        class_bits: 0,
        serverclass_map: HashMap::default(),
        entities: Some(HashMap::default()),
        bad: Vec::new(),
        stringtables: Vec::new(),
        players: Vec::new(),
        parse_props: false,
        game_events: Vec::new(),
        event_name: "".to_string(),
        wanted_props: Vec::new(),
        cnt: 0,
    };
    use std::time::Instant;
    let now = Instant::now();

    let h: Header = d.parse_header();
    let mut event_names: Vec<String> = Vec::new();

    let data = d.parse_frame(&props_names);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    println!("{}", d.cnt);
}
