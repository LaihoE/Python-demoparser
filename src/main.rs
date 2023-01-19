mod parsing;
use crate::parsing::game_events::GameEvent;
use ahash::RandomState;
use csgoproto::netmessages;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use csv::Writer;
use memmap2::Mmap;
use memmap2::MmapOptions;
use mimalloc::MiMalloc;
use parsing::game_events::KeyData;
use parsing::header::Header;
use parsing::parser::Parser;
use parsing::variants::VarVec;
use phf::phf_map;
use polars::prelude::ArrowField;
use polars::prelude::NamedFrom;
use polars::series::Series;
use polars_arrow::export::arrow;
use polars_arrow::prelude::ArrayRef;
use protobuf;
use protobuf::reflect::MessageDescriptor;
use protobuf::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::time::Instant;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub fn max_skip_tick(game_events: &Vec<GameEvent>) -> i32 {
    let mut biggest_needed_tick = 0;
    for ge in game_events {
        if ge.name == "player_connect_full" {
            for field in &ge.fields {
                match field.name.as_str() {
                    "tick" => {
                        if let Some(KeyData::Long(t)) = field.data {
                            biggest_needed_tick = t;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    //println!("Biggest needed {}", biggest_needed_tick);
    biggest_needed_tick
}

fn main() {
    let now = Instant::now();
    let paths = fs::read_dir("/home/laiho/Documents/demos/benchmark/").unwrap();
    let p = "/home/laiho/Documents/demos/faceits/m/1-0e7a456a-6318-4f85-ae08-e203823e1758_76561199053401103.dem";

    for demo_path in paths {
        let now = Instant::now();
        let props_names = vec!["m_vecOrigin".to_string()];
        println!("{:?}", demo_path.as_ref().unwrap().path());
        let mut parser = Parser::new(
            demo_path
                .unwrap()
                .path()
                .as_path()
                .to_str()
                .unwrap()
                .to_string(),
            true,
            vec![],
            vec![],
            vec!["m_angEyeAngles[0]".to_string()],
            "player_death".to_string(),
            false,
            false,
            false,
            1000000,
            props_names.clone(),
        )
        .unwrap();

        let h: Header = parser.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let data_a = parser.start_parsing(&props_names);
        //break;
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }
    // 145
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 67);
}
