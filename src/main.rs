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
    let paths = fs::read_dir("/home/laiho/Documents/demos/broken/").unwrap();
    //let paths = fs::read_dir("/home/laiho/Documents/demos/mygames/").unwrap();
    for demo_path in paths {
        //let now = Instant::now();
        let props_names = vec!["m_angEyeAngles[0]".to_string()];
        let x = netmessages::file_descriptor();
        let y = x.messages();
        println!("{:?}", demo_path.as_ref().unwrap().path());
        let file = File::open(demo_path.unwrap().path()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let mut parser = Demo::new_mmap(
            mmap,
            true,
            (50..150).collect(),
            vec![76561198194694750],
            vec![],
            "player_death".to_string(),
            false,
            false,
        )
        .unwrap();

        let h: Header = parser.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let data = parser.start_parsing(&props_names);

        // 41 ent   39 rules
        for ent in parser.entities {
            if ent.1.class_id == 41 {
                for p in ent.1.props {
                    if p.0.contains("m_iCompetitiveRanking") {
                        println!("{:?}", p)
                    }
                }
            }
        }
        break;
        //break;
    }
    let elapsed = now.elapsed();
    // Elapsed: 13.30s (avg: 189.98ms)

    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 70);
}
