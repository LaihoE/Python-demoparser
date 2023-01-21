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
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::time::Instant;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn parse_demo(demo_path: String) -> i32 {
    // m_iHealth
    // m_angEyeAngles[1]
    println!("{}", demo_path);

    let now = Instant::now();
    let props_names = vec!["m_angEyeAngles[1]".to_string()];

    let mut parser = Parser::new(
        demo_path,
        true,
        vec![],
        vec![],
        vec!["m_angEyeAngles[1]".to_string()],
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
    parser.settings.playback_frames = (h.playback_ticks + 100) as usize;
    parser.start_parsing(&props_names);
    let elapsed = now.elapsed();
    //println!("Elapsed: {:.2?} ", elapsed);
    69
}

fn main() {
    let now = Instant::now();
    let paths = fs::read_dir("/home/laiho/Documents/demos/mygames/").unwrap();
    let mut paths_v = vec![];
    for path in paths {
        let p = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        paths_v.push(p);
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(12)
        .build_global()
        .unwrap();
    println!("{:?}", paths_v.len());

    let this_p = &paths_v[0];
    let single = vec![this_p];
    let x: Vec<i32> = single
        .into_iter()
        .map(|f| parse_demo(f.to_owned()))
        .collect();
    // 145
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 67);
}
