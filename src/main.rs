mod parsing;
use crate::parsing::game_events::GameEvent;
use ahash::RandomState;
use csgoproto::netmessages;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use csv::Writer;
use memmap::Mmap;
use memmap::MmapOptions;
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
    let paths = fs::read_dir("/home/laiho/Documents/demos/faceits/cu/").unwrap();
    for demo_path in paths {
        let now = Instant::now();
        let props_names = vec!["m_vecOrigin".to_string()];
        let dp = "/home/laiho/Documents/demos/mygames/aa.dem".to_string();
        println!("{:?}", demo_path.as_ref().unwrap().path());
        let mut parser = Parser::new(
            demo_path
                .as_ref()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            true,
            (50..70000).collect(),
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
        let data = parser.start_parsing(&props_names);
        /*
        for (k, v) in data {
            for (kk, vv) in v {
                match vv {
                    VarVec::F32(v) => {
                        let s = Series::new("oogla", v);
                        for x in &s.0 {
                            println!("{}", x);
                        }
                    }
                    _ => {}
                }
            }
        }
        */
        //let x = data.read().unwrap();
        //println!("{:?}", x);
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 1000);
}
