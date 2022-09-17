mod parsing;
use parsing::header::Header;
use parsing::parser::Demo;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::Instant;

fn main() {
    let demo_path = "/home/laiho/Documents/demos/rclonetest/w.dem";
    let props_names = vec!["m_vecOrigin_X".to_string()];

    let mut d = Demo {
        bytes: std::fs::read(demo_path).unwrap(),
        fp: 0,
        cmd: 0,
        tick: 0,
        event_list: None,
        event_vec: None,
        dt_map: Some(HashMap::new()),
        class_bits: 0,
        serverclass_map: HashMap::new(),
        entities: Some(HashMap::new()),
        bad: Vec::new(),
        stringtables: Vec::new(),
        players: Vec::new(),
        parse_props: true,
        game_events: Vec::new(),
        event_name: "".to_string(),
    };

    let h: Header = d.parse_header();
    let mut event_names: Vec<String> = Vec::new();

    let data = d.parse_frame(&props_names);
}
