mod parsing;
use mimalloc::MiMalloc;
use parsing::parser::Parser;
use parsing::utils::Header;
use rayon::prelude::IntoParallelIterator;
use std::fs;
use std::time::Instant;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
    let props_names = vec!["manager_m_iKills".to_string()];

    let mut parser = Parser::new(
        demo_path,
        true,
        false,
        //vec![],
        (10000..10002).collect(),
        vec![],
        vec!["manager_m_iKills".to_string()],
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
    //let paths = fs::read_dir("/home/laiho/Documents/demos/faceits/cu/").unwrap();
    let paths = fs::read_dir("/home/laiho/Documents/demos/mygames/").unwrap();
    //let paths = fs::read_dir("/media/laiho/cc302116-f9ac-4408-a786-7c7df3e7d807/dems").unwrap();

    let mut paths_v = vec![];
    for path in paths {
        let p = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        paths_v.push(p);
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build_global()
        .unwrap();

    let this_p = &paths_v[0];
    let single = vec![this_p];
    /*
    let x: Vec<i32> = tqdm!(paths_v.into_iter())
        .map(|f| parse_demo(f.to_owned()))
        .collect();
    */
    use rayon::iter::ParallelIterator;

    let x: Vec<i32> = paths_v
        .into_par_iter()
        .map(|f| parse_demo(f.to_string()))
        .collect();

    // 145
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 315);
    // /home/laiho/Documents/demos/faceits/cu/003309131115255562271_1824323488 (1).dem
    /*
    pub enum KeyData{
        Str(String),
        Float(f32),
        Long(i32),
        Short(i16),
        Byte(u8),
        Bool(bool),
        Uint64(u64),
    }
    fn enum_vec_to_vec(all_keydata: Vec<KeyData>, wanted_type: KeyData::Float) -> Vec<?> {
        let v = vec![];
        for kd in all_keydata {
            if let KeyData::Float(l) = kd {
                v.push(l)
            }
        }
        v
    }

    fn enum_vec_to_FLOAT_vec(all_variants: Vec<Variant>) -> Vec<f32> {
        let v = vec![];
        for variant in all_variants {
            if let Variant::Float(f) = variant{
                v.push(f)
            }
        }
        v
    }
    */
}
