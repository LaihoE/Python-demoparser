mod parsing;
use crate::parsing::game_events::GameEvent;
use ahash::RandomState;
use csgoproto::netmessages;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use csv::Writer;
use fxhash::FxHashMap;
use memmap::Mmap;
use memmap::MmapOptions;
use parsing::game_events::KeyData;
use parsing::header::Header;
use parsing::parser::Parser;
use phf::phf_map;
use protobuf;
use protobuf::reflect::MessageDescriptor;
use protobuf::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::time::Instant;

pub fn main() {
    let now = Instant::now();
    let parser = Parser::new(
        "/home/laiho/Documents/demos/mygames/match730_003439547603925074007_0749396926_184.dem"
            .to_string(),
        false,
        vec![],
        vec![],
        vec!["m_vecOrigin_X".to_string()],
        "player_death".to_string(),
        false,
        false,
        false,
        9999999,
        vec!["m_vecOrigin_X".to_string()],
    );
    match parser {
        Err(e) => panic!("{}", e),
        Ok(mut parser) => {
            let _: Header = parser.parse_demo_header();
            let (_, mut tc) = parser.start_parsing(&vec!["m_iMVPs".to_owned()]);

            tc.gather_eventprops_backwards(
                &mut parser.state.game_events,
                vec!["m_vecOrigin_X".to_string()],
                &parser.bytes,
                &parser.maps.baselines,
                &parser.maps.serverclass_map,
                &parser.maps.userid_sid_map,
                &parser.maps.uid_eid_map,
            );
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
