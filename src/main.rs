mod parsing;
use mimalloc::MiMalloc;
use ndarray::{arr1, Array1};
use parsing::parser::Parser;
use parsing::utils::Header;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator};
use std::fs;
use std::time::Instant;

use std::env;
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
    let props_names = vec![];

    //let wanted_ticks = vec![10000]

    let mut parser = Parser::new(
        demo_path,
        true,
        false,
        //vec![],
        (0..10000).collect(),
        vec![],
        vec!["player@m_vecOrigin_X".to_string()],
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
    parser.start_parsing();
    let elapsed = now.elapsed();
    //println!("Elapsed: {:.2?} ", elapsed);
    69
}

fn main() {
    let now = Instant::now();
    //let paths = fs::read_dir("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems/").unwrap();
    let paths = fs::read_dir("/home/laiho/Documents/demos/faceits/cu/").unwrap();
    //let paths = fs::read_dir("/home/laiho/Documents/demos/mygames/").unwrap();
    //let paths = fs::read_dir("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems").unwrap();
    //let paths = fs::read_dir("/home/laiho/Documents/demos/bench_pro_demos/").unwrap();

    let mut paths_v = vec![];
    for path in paths {
        let p = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        paths_v.push(p);
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(12)
        .build_global()
        .unwrap();

    let this_p = &paths_v[0];
    let single = vec![this_p];

    use rayon::iter::ParallelIterator;
    let x: Vec<i32> = single.iter().map(|f| parse_demo(f.to_string())).collect();

    // 145
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 315);
}
