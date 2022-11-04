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
    println!("Biggest needed {}", biggest_needed_tick);
    biggest_needed_tick
}

fn main() {
    let now = Instant::now();
    let paths = fs::read_dir("/home/laiho/Documents/demos/faceits/cu/").unwrap();
    for demo_path in paths {
        let now = Instant::now();
        let props_names = vec!["m_vecOrigin".to_string()];
        let dp = "/home/laiho/Documents/demos/mygames/aa.dem".to_string();
        let mut parser = Demo::new(
            demo_path
                .as_ref()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            false,
            (50..70000).collect(),
            vec![],
            vec!["m_angEyeAngles[0]".to_string()],
            "".to_string(),
            false,
            false,
            false,
            1000000,
            props_names.clone(),
        )
        .unwrap();

        let h: Header = parser.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let (data, mut tc) = parser.start_parsing(&props_names);

        //let elapsed = now.elapsed();

        let kill_ticks = get_event_md(&parser.game_events, &parser.sid_entid_map);
        //println!("{:?}", kill_ticks);

        let m = max_skip_tick(&parser.game_events);

        let mut parser2 = Demo::new(
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
            "".to_string(),
            false,
            false,
            true,
            m,
            props_names.clone(),
        )
        .unwrap();

        let h: Header = parser2.parse_demo_header();
        let mut event_names: Vec<String> = Vec::new();
        let (data, _) = parser2.start_parsing(&props_names);

        let mut total = 0;
        let mut cur_tick = 0;
        for event_md in kill_ticks {
            cur_tick = event_md.tick;

            if cur_tick <= m {
                continue;
            }

            'outer: loop {
                total += 1;
                match tc.get_tick_inxes(cur_tick as usize) {
                    Some(inxes) => {
                        let bs = &parser.bytes[inxes.0..inxes.1];
                        let msg = Message::parse_from_bytes(bs).unwrap();
                        let d = tc.parse_packet_ents_simple(
                            msg,
                            &parser2.entities,
                            &parser2.serverclass_map,
                        );
                        match d.get(&event_md.player) {
                            Some(x) => {
                                for i in x {
                                    if i.0 == "m_vecOrigin_X" {
                                        /*
                                        println!(
                                            "Delta found at tick: {} start: {} val: {:?} ent:{:?}",
                                            cur_tick, event_md.tick, i.1, &event_md.player
                                        );
                                        */
                                        break 'outer;
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    None => {}
                }
                cur_tick -= 1;
                if cur_tick <= m {
                    //panic!("tick {}", cur_tick);
                    break;
                }
            }
        }
        //let elapsed = println!("tick {}", parser2.tick);
        println!("Total ticks parsed: {}", total);
        let elapsed = now.elapsed();
        //println!("Elapsed: {:.2?} (avg: {:.2?})");
        println!("Elapsed: {:.2?}", elapsed);
        //break;
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 67);
}

#[derive(Debug, Clone)]
pub struct EventMd {
    pub tick: i32,
    pub player: u32,
    pub attacker: Option<u32>,
}

pub fn get_event_md(game_events: &Vec<GameEvent>, sid_eid_map: &HashMap<u64, u32>) -> Vec<EventMd> {
    let mut md = vec![];

    for event in game_events {
        if event.name == "player_death" {
            let mut player = 420000000;
            let mut attacker = Default::default();
            let mut tick = -10000;

            for f in &event.fields {
                if f.name == "tick" {
                    match f.data.as_ref().unwrap() {
                        KeyData::Long(x) => {
                            tick = *x;
                        }
                        _ => {}
                    }
                }
                if f.name == "player_steamid" {
                    match f.data.as_ref().unwrap() {
                        KeyData::Uint64(x) => match sid_eid_map.get(x) {
                            Some(eid) => player = *eid,
                            None => panic!("no ent found for sid"),
                        },
                        _ => {}
                    }
                }
                if f.name == "attacker_steamid" {
                    match f.data.as_ref().unwrap() {
                        KeyData::Uint64(x) => match sid_eid_map.get(x) {
                            Some(eid) => attacker = Some(*eid),
                            None => panic!("no ent found for sid"),
                        },
                        _ => {}
                    }
                }
            }
            md.push(EventMd {
                tick,
                player,
                attacker,
            })
        }
    }
    md
}
