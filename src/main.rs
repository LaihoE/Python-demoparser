mod parsing;
use ahash::RandomState;
use csgoproto::netmessages;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use csv::Writer;
use memmap::Mmap;
use memmap::MmapOptions;
use parsing::header::Header;
use parsing::parser::Demo;
use phf::phf_map;
use protobuf;
use protobuf::reflect::MessageDescriptor;
use protobuf::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::time::Instant;

fn main() {
    let now = Instant::now();
    let paths = fs::read_dir("/home/laiho/Documents/demos/faceits/average/").unwrap();
    for demo_path in paths {
        let now = Instant::now();
        let props_names = vec!["m_angEyeAngles[0]".to_string()];
        let dp = "/home/laiho/Documents/demos/mygames/ww.dem".to_string();
        let mut parser = Demo::new(
            /*
            demo_path
                .as_ref()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            */
            dp,
            true,
            (50..70000).collect(),
            vec![],
            vec!["m_angEyeAngles[0]".to_string()],
            "".to_string(),
            false,
            false,
            false,
            1000000,
            props_names.clone(),
        )
        .unwrap();

        let h: Header = parser.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let data = parser.start_parsing(&props_names);

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        break;
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 67);
}
