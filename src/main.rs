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

fn main() {
    let now = Instant::now();
    let paths = fs::read_dir("/home/laiho/Documents/demos/faceits/average/").unwrap();
    for demo_path in paths {
        let now = Instant::now();
        let props_names = vec!["m_vecOrigin".to_string()];
        let dp = "/home/laiho/Documents/demos/mygames/c.dem".to_string();
        let mut parser = Demo::new(
            /*
            demo_path
                .as_ref()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            */
            dp,
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
        let (data, mut tc) = parser.start_parsing(&props_names);

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        let kill_ticks = get_event_md(&parser.game_events, &parser.sid_entid_map);
        //println!("{:?}", kill_ticks);

        for (k, v) in parser.uid_eid_map {
            println!("{} {}", k, v);
        }
        let mut total = 0;
        let mut cur_tick = 0;
        for event_md in kill_ticks {
            cur_tick = event_md.tick;
            if cur_tick < 5000 {
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
                            &parser.entities,
                            &parser.serverclass_map,
                        );
                        match d.get(&event_md.player) {
                            Some(x) => {
                                for i in x {
                                    if i.0 == 10000 {
                                        println!(
                                            "Delta found at tick: {} start: {} val: {:?} ent:{:?}",
                                            cur_tick, event_md.tick, i.1, &event_md.player
                                        );
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
                if cur_tick < 5000 {
                    //panic!("tick {}", cur_tick);
                    break;
                }
            }
        }
        println!("Total ticks parsed: {}", total);

        break;
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
