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
    let paths = fs::read_dir("/mnt/d/b/mygames/").unwrap();
    for demo_path in paths {
        //let paths = fs::read_dir("/mnt/d/b/mygames/").unwrap();

        let now = Instant::now();
        let props_names = vec!["m_angEyeAngles[0]".to_string()];
        let dp = "/mnt/d/b/mygames/".to_string();
        let mut parser = Demo::new(
            demo_path
                .as_ref()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            true,
            (50..100).collect(),
            vec![],
            vec!["m_angEyeAngles[1]".to_string(), "m_iHealth".to_string()],
            "".to_string(),
            false,
            false,
            false,
            1000000,
            vec![],
        )
        .unwrap();

        let h: Header = parser.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let data = parser.start_parsing(&props_names);
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);

        /*
        for (k, v) in parser.entities {
            //println!("{}", parser.serverclass_map[&(v.class_id as u16)].dt);

            if v.class_id == 41 {
                for (x, y) in v.props {
                    println!("{} {:?}", x, y.data);
                }
            }
        }
        */
        //println!("Elapsed: {:.2?}", elapsed);
        //break;
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 67);
}
