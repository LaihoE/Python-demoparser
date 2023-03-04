mod parsing;
use crate::parsing::parser_settings::ParserInputs;
use itertools::Itertools;
use mimalloc::MiMalloc;
use ndarray::{arr1, Array1};
use parsing::parser::Parser;
use parsing::utils::Header;
use parsing::utils::CACHE_ID_MAP;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator};
use std::env;
use std::fs;
use std::time::Instant;

fn x() {
    // this method needs to be inside main() method
    env::set_var("RUST_BACKTRACE", "1");
}

fn parse_demo(demo_path: String) -> i32 {
    // m_iHealth
    // m_angEyeAngles[1]
    println!("{}", demo_path);

    if demo_path
        == "/home/laiho/Documents/demos/faceits/cu/003309131115255562271_1824323488 (1).dem"
    {
        return 69;
    }

    let now = Instant::now();

    //let wanted_ticks = vec![10000]
    let parser_inputs = ParserInputs {
        demo_path: demo_path,
        parse_props: false,
        only_events: true,
        wanted_ticks: vec![],
        wanted_players: vec![],
        event_name: "bomb_planted".to_string(),
        only_players: true,
        parse_game_events: true,
        og_names: vec![],
        collect_props: vec![],
        wanted_props: vec![],
    };
    let mut parser = Parser::new(parser_inputs).unwrap();

    let h: Header = parser.parse_demo_header();
    let mut event_names: Vec<String> = Vec::new();
    parser.settings.playback_frames = (h.playback_ticks + 100) as usize;
    parser.start_parsing();
    let elapsed = now.elapsed();
    //println!("Elapsed: {:.2?} ", elapsed);
    69
}

fn main() {
    let now = Instant::now();
    //let paths = fs::read_dir("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems/").unwrap();
    //let paths = fs::read_dir("/home/laiho/Documents/demos/faceits/cu/").unwrap();
    //let paths = fs::read_dir("/home/laiho/Documents/demos/mygames/").unwrap();
    //let paths = fs::read_dir("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems").unwrap();
    let paths = fs::read_dir("/home/laiho/Documents/demos/bench_pro_demos/").unwrap();

    let mut paths_v = vec![];
    for path in paths {
        let p = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        paths_v.push(p);
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(12)
        .build_global()
        .unwrap();

    let this_p = &paths_v[8];
    let single = vec![this_p];

    use rayon::iter::ParallelIterator;
    let x: Vec<i32> = paths_v.iter().map(|f| parse_demo(f.to_string())).collect();

    // 145
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 315);
}
