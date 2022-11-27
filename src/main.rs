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

        let kill_ticks = get_event_md(&parser.state.game_events, &parser.maps.sid_entid_map);
        let m = max_skip_tick(&parser.state.game_events);
        //println!("{}", m);
        if m > 20000 {
            println!("BREAK");
            continue;
        }

        //println!("{:?}", parser.sid_entid_map);
        //let elapsed = println!("tick {}", parser2.tick);
        //println!("Total ticks parsed: {}", total);
        let elapsed = now.elapsed();
        //println!("Elapsed: {:.2?} (avg: {:.2?})");
        println!("Elapsed: {:.2?}", elapsed);
        //break;
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} (avg: {:.2?})", elapsed, elapsed / 1000);
}

#[derive(Debug, Clone)]
pub struct EventMd {
    pub tick: i32,
    pub player: u32,
    pub attacker: Option<u32>,
}

pub fn get_current_entid(
    tick: &i32,
    sid: &u64,
    sid_eid_map: &HashMap<u64, Vec<(u32, i32)>>,
) -> u32 {
    match sid_eid_map.get(&sid) {
        None => {
            //panic!("No entid for steamid")
            return 0;
        }
        Some(tups) => {
            if tups.len() == 1 {
                return tups[0].0;
            }
            //println!(">1: {:?}", tups);
            for t in 0..tups.len() - 1 {
                if tups[t + 1].1 > *tick && tups[t].1 < *tick {
                    //println!("tick: {} returned: {}", tick, tups[t].0);
                    return tups[t].0;
                }
            }
            if tups[tups.len() - 1].1 < *tick {
                //println!("tick: {} returned: {}", tick, tups[tups.len() - 1].0);
                return tups[tups.len() - 1].0;
            }
            return tups[0].0;
        }
    };
}

pub fn get_event_md(
    game_events: &Vec<GameEvent>,
    sid_eid_map: &HashMap<u64, Vec<(u32, i32)>>,
) -> Vec<EventMd> {
    let mut md = vec![];
    for event in game_events {
        if event.name == "player_death" {
            //println!("{:?}", event);
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
                    //println!("player: {:?}", f.data);
                    match f.data.as_ref().unwrap() {
                        KeyData::Uint64(x) => player = *x,
                        _ => {}
                    }
                }
                if f.name == "attacker_steamid" {
                    //println!("attacker: {:?}", f.data);
                    match f.data.as_ref().unwrap() {
                        KeyData::Uint64(x) => attacker = Some(x),
                        _ => {}
                    }
                }
            }
            let attacker_id = match attacker {
                Some(sid) => Some(get_current_entid(&tick, sid, sid_eid_map)),
                None => None,
            };
            let player_id = get_current_entid(&tick, &player, sid_eid_map);
            md.push(EventMd {
                tick,
                player: player_id,
                attacker: attacker_id,
            })
        }
    }
    md
}
