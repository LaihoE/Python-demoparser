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
    let paths = fs::read_dir("/home/laiho/Documents/demos/mygames/").unwrap();
    for demo_path in paths {
        let props_names = vec![
            "m_angEyeAngles[0]".to_string(),
            "m_iCompetitiveRanking".to_string(),
        ];

        let mut parser = Demo::new(
            demo_path
                .as_ref()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            true,
            vec![],
            vec![],
            vec![],
            "".to_string(),
            true,
            false,
        )
        .unwrap();

        let h: Header = parser.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let data = parser.start_parsing(&props_names);
        //println!("{:?}", parser.manager_id);

        // 41 ent   39 rules

        for ent in parser.entities {
            if ent.1.class_id == 41 {
                for p in ent.1.props {
                    println!("X {:?}", p.1);
                }
            }
        }

        break;
    }
    let elapsed = now.elapsed();
    // Elapsed: 13.30s (avg: 189.98ms)
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 70);
}
