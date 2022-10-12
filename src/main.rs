mod parsing;
use csgoproto::netmessages;
use memmap::MmapOptions;
use parsing::header::Header;
use parsing::parser::Demo;
use protobuf::reflect::MessageDescriptor;
use std::fs;
use std::fs::File;
use std::time::Instant;

fn main() {
    let now = Instant::now();
    //let paths = fs::read_dir("/home/laiho/Documents/demos/faceits/test/").unwrap();
    let paths = fs::read_dir("/media/laiho/New Volume1/b/b").unwrap();
    for demo_path in paths {
        //let demo_path = "/home/laiho/Documents/demos/faceits/clean_unzompr/1.dem";
        let props_names = vec!["m_angEyeAngles[0]".to_string()];
        let x = netmessages::file_descriptor();
        let y = x.messages();
        let mut v: Vec<MessageDescriptor> = Vec::new();
        let mut cnt = 0;
        println!("{:?}", demo_path.as_ref().unwrap().path());
        let file = File::open(demo_path.unwrap().path()).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let mut parser = Demo::new_mmap(
            mmap,
            true,
            (50..100).collect(),
            vec![],
            vec![],
            "".to_string(),
            false,
            false,
        )
        .unwrap();

        let h: Header = parser.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let data = parser.start_parsing(&props_names);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 35);
}
