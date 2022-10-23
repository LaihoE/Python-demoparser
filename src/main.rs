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
    let paths = fs::read_dir("/mnt/d/b/mygames/").unwrap();
    for demo_path in paths {
        let now = Instant::now();
        let props_names = vec!["m_angEyeAngles[0]".to_string()];

        let mut parser = Demo::new(
            demo_path
                .as_ref()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            true,
            vec![10000, 10001, 10002, 10003],
            vec![],
            vec![],
            "".to_string(),
            false,
            false,
            true,
            50
        )
        .unwrap();

        let h: Header = parser.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let data = parser.start_parsing(&props_names);

        // 41 ent   39 rules
        /*
        for ent in parser.entities {
            if ent.1.class_id == 39 {
                println!("{:?}", ent);
                for p in ent.1.props {
                    println!("X {:?}", p.1);
                }
            }
        }
        */
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        //break;
    }

    //println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 70);
}
