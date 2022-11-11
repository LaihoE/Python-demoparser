use super::{
    data_table::ServerClass,
    entities,
    variants::create_default_from_pdata,
    variants::{PropData, VarVec},
};
use crate::parsing::entities::Entity;
use crate::parsing::read_bits::MyBitreader;
use crate::parsing::variants::PropAtom;
use ahash::RandomState;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use phf::phf_map;
use protobuf::Message;
use std::collections::HashMap;
pub struct TickCache {
    ticks: Vec<(usize, usize)>,
    pub ents: HashMap<u32, HashMap<String, VarVec>>,
}
use crate::parsing::game_events::NameDataPair;
use crate::parsing::parser::Parser;
use crate::parsing::variants::BytesVariant;
use crate::GameEvent;
use crate::KeyData;

#[derive(Debug, Clone)]
pub struct EventMd {
    pub tick: i32,
    pub player_eid: u32,
    pub attacker_eid: Option<u32>,
    pub player_sid: u64,
    pub attacker_sid: Option<u64>,
    pub player_min_tick: i32,
    pub attacker_min_tick: i32,
}

impl TickCache {
    pub fn new() -> Self {
        let mut t: Vec<(usize, usize)> = vec![];
        for i in 0..1000000 {
            t.push((0, 0));
        }
        TickCache {
            ticks: t,
            ents: HashMap::default(),
        }
    }

    pub fn get_prop_at_tick(&self, tick: i32, prop_inx: String, ent_id: u32) -> Option<PropData> {
        match self.ents.get(&ent_id) {
            Some(u) => match u.get(&prop_inx) {
                Some(var_v) => match var_v {
                    VarVec::F32(f) => match f.get(tick as usize) {
                        Some(val) => match val {
                            Some(x) => return Some(PropData::F32(*x)),
                            None => None,
                        },
                        None => return None,
                    },
                    VarVec::I32(f) => match f.get(tick as usize) {
                        Some(val) => match val {
                            Some(x) => return Some(PropData::I32(*x)),
                            None => None,
                        },
                        None => return None,
                    },
                    VarVec::String(f) => match f.get(tick as usize) {
                        Some(val) => match val {
                            Some(x) => return Some(PropData::String(x.clone())),
                            None => None,
                        },
                        None => return None,
                    },
                    _ => None,
                },
                None => return None,
            },
            None => return None,
        }
    }
    pub fn insert_tick(&mut self, tick: i32, left: usize, right: usize) {
        // Tick indicies in bytes. Could also be ref to bytes
        self.ticks[tick as usize] = (left, right);
    }
    pub fn insert_cache(&mut self, tick: i32, prop_inx: String, prop: PropData, ent_id: u32) {
        // insert already parsed ticks into cache so that we don't
        // parse the same stuff multiple times
        match prop {
            PropData::Vec(_) => return,
            PropData::VecXY(_) => return,
            PropData::VecXYZ(_) => return,
            _ => {}
        }
        match self.ents.get_mut(&ent_id) {
            Some(e) => match e.get_mut(&prop_inx) {
                Some(v) => {
                    v.insert_propdata(tick as usize, prop);
                }
                None => {
                    e.insert(prop_inx, create_default_from_pdata(prop, 500000));
                    // Bug watch out
                    // v.insert_propdata(tick as usize, prop);
                }
            },
            None => {
                self.ents.insert(ent_id, HashMap::default());
            }
        }
    }
    pub fn insert_cache_multiple(&mut self, tick: i32, hm: &HashMap<u32, Vec<(String, PropData)>>) {
        for (player, data) in hm {
            for (prop_inx, p) in data {
                self.insert_cache(tick, prop_inx.to_string(), p.clone(), *player)
            }
        }
    }
    pub fn get_tick_inxes(&self, inx: usize) -> Option<(usize, usize)> {
        match self.ticks.get(inx) {
            Some(t) => {
                if t.0 == 0 && t.1 == 0 {
                    return None;
                }
                Some(t.clone())
            }
            None => None,
        }
    }

    pub fn gather_eventprops_backwards(
        &mut self,
        game_events: &mut Vec<GameEvent>,
        wanted_props: Vec<String>,
        bytes: &BytesVariant,
        baselines: &HashMap<u32, HashMap<String, PropData>>,
        serverclass_map: &HashMap<u16, ServerClass, RandomState>,
        userid_sid_map: &HashMap<u32, Vec<(u64, i32)>, RandomState>,
        uid_eid_map: &HashMap<u32, Vec<(u32, i32)>, RandomState>,
    ) {
        /*
        Function for gathering props at given events.
        Does this by starting from wanted tick and going
        backwards until it finds the latest "delta".
        */
        let mut ge_inx = -1;
        let mut cur_tick = 0;
        let mut entities = vec![];
        let event_mds = get_event_md(game_events, userid_sid_map, uid_eid_map);
        let mut tot = 0;
        for i in 0..100 {
            entities.push((
                1111111,
                Entity {
                    class_id: 40,
                    entity_id: 1111111,
                    props: HashMap::default(),
                },
            ));
        }
        for event_md in &event_mds {
            let mut wanted_props_player = wanted_props.clone();
            // If event has attacker then add props
            // loop brakes on player.empty() && attacker.empty
            let mut wanted_props_attacker: Vec<String> = if event_md.attacker_eid.is_some() {
                wanted_props.clone()
            } else {
                vec![]
            };
            let mut subtot = 0;
            ge_inx += 1;
            cur_tick = event_md.tick - 1;
            loop {
                subtot += 1;
                if wanted_props_player.is_empty() && wanted_props_attacker.is_empty() {
                    //println!("{}", subtot);
                    break;
                }

                if event_md.attacker_sid.is_some() {
                    if event_md.attacker_sid.unwrap() == 0 {
                        break;
                    }
                }

                if cur_tick < -50 {
                    /*
                    println!(
                        "out: {} player:{}, attacker:{:?} {:?} {:?}",
                        event_md.tick,
                        event_md.player_eid,
                        event_md.attacker_eid,
                        event_md.attacker_sid,
                        event_md.player_sid
                    );
                    */
                    break;
                }
                match self.get_tick_inxes(cur_tick as usize) {
                    Some(inxes) => {
                        tot += 1;
                        let msg = Message::parse_from_bytes(&bytes[inxes.0..inxes.1]).unwrap();
                        let this_tick_deltas = self.parse_packet_ents_simple(
                            msg,
                            &mut entities,
                            serverclass_map,
                            baselines,
                            cur_tick,
                        );
                        // Props for player

                        match this_tick_deltas.get(&event_md.player_eid) {
                            Some(x) => {
                                for i in x {
                                    if wanted_props.contains(&i.0) {
                                        wanted_props_player.retain(|x| *x != i.0);
                                        game_events[ge_inx as usize].fields.push(NameDataPair {
                                            name: "player_".to_string() + &i.0,
                                            data: Some(KeyData::from_pdata(&i.1)),
                                        });
                                    }
                                }
                            }
                            None => {}
                        }

                        if event_md.attacker_eid.is_some() {
                            match this_tick_deltas.get(&event_md.attacker_eid.unwrap()) {
                                Some(x) => {
                                    for i in x {
                                        if wanted_props.contains(&i.0) {
                                            wanted_props_attacker.retain(|x| *x != i.0);
                                            game_events[ge_inx as usize].fields.push(
                                                NameDataPair {
                                                    name: "attacker_".to_string() + &i.0,
                                                    data: Some(KeyData::from_pdata(&i.1)),
                                                },
                                            );
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                    None => {}
                }
                cur_tick -= 1;
            }
        }
        //println!("{}", tot);
    }
    // Stripped down version of the "real" parse packet ents
    pub fn parse_packet_ents_simple(
        &mut self,
        pack_ents: CSVCMsg_PacketEntities,
        entities: &mut Vec<(u32, Entity)>,
        serverclass_map: &HashMap<u16, ServerClass, RandomState>,
        baselines: &HashMap<u32, HashMap<String, PropData>>,
        tick: i32,
    ) -> HashMap<u32, Vec<(String, PropData)>> {
        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut entity_id: i32 = -1;

        let mut updated_vals: HashMap<u32, Vec<(String, PropData)>> = HashMap::default();

        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var().unwrap() as i32);

            if entity_id > 50 {
                break;
            }
            if b.read_boolie().unwrap() {
                b.read_boolie().unwrap();
            } else if b.read_boolie().unwrap() {
                let cls_id = b.read_nbits(9);
                let _ = b.read_nbits(10);
                /*
                panic!(
                    "Tried to create new ent in speedy mode. Entid: {}",
                    entity_id
                );
                */
                //break;

                let mut val = -1;
                let new_way = b.read_boolie().unwrap();
                let mut v = vec![];
                loop {
                    val = b.read_inx(val, new_way).unwrap();

                    if val == -1 {
                        break;
                    }
                    v.push(val)
                }
                let this_v = updated_vals.entry(entity_id as u32).or_insert(vec![]);
                //println!("DT: {}", serverclass_map[&(cls_id.unwrap() as u16)].dt);

                for inx in v {
                    let prop = &serverclass_map[&(cls_id.unwrap() as u16)].props[inx as usize];
                    let pdata = b.decode(prop).unwrap();

                    match pdata {
                        PropData::VecXY(v) => {
                            let endings = ["_X", "_Y"];
                            for inx in 0..2 {
                                let data = PropData::F32(v[inx]);
                                this_v.push((prop.name.to_string() + endings[inx], data));
                            }
                        }
                        PropData::VecXYZ(v) => {}
                        _ => {
                            this_v.push((prop.name.to_string(), pdata));
                        }
                    }
                }
            } else {
                match entities.get(entity_id as usize) {
                    Some(_) => {}
                    None => {
                        let mut e = if entity_id == 1 {
                            Entity {
                                // Cls id for bot
                                class_id: 40,
                                entity_id: entity_id as u32,
                                props: HashMap::default(),
                            }
                        } else if entity_id == 0 {
                            Entity {
                                // Cls id for bot
                                class_id: 275,
                                entity_id: entity_id as u32,
                                props: HashMap::default(),
                            }
                        } else {
                            Entity {
                                // Cls id for player
                                class_id: 40,
                                entity_id: entity_id as u32,
                                props: HashMap::default(),
                            }
                        };
                        match baselines.get(&e.class_id) {
                            Some(baseline) => {
                                for (k, v) in baseline {
                                    let atom = PropAtom {
                                        prop_name: k.to_string(),
                                        data: v.clone(),
                                        tick: 22,
                                    };
                                    e.props.insert(k.to_string(), atom);
                                }
                            }
                            None => {}
                        }
                        entities[entity_id as usize] = (entity_id as u32, e);
                    }
                };
                let ent = &entities[entity_id as usize];
                /*
                let sv_cls = if entity_id <= 10 {
                    &serverclass_map[&(40 as u16)]
                } else {
                    &serverclass_map[&(1 as u16)]
                };
                */
                let sv_cls = &serverclass_map[&(ent.1.class_id as u16)];
                /*
                if sv_cls.dt != "DT_CSPlayer" {
                    println!(
                        "BOT SPOTTED eid {}, dt:{} !!!! Tick:{}",
                        entity_id, sv_cls.dt, tick
                    );
                    //println!("XXX {}", sv_cls.id);
                    //println!("NOT PLAYER: {}, TYPE: {}", entity_id, sv_cls.dt);
                    //break;
                }
                */
                let mut val = -1;
                let new_way = b.read_boolie().unwrap();
                let mut v = vec![];
                loop {
                    val = b.read_inx(val, new_way).unwrap();

                    if val == -1 {
                        break;
                    }
                    v.push(val)
                }
                let this_v = updated_vals.entry(entity_id as u32).or_insert(vec![]);

                for inx in v {
                    let prop = &sv_cls.props[inx as usize];
                    let pdata = b.decode(prop).unwrap();

                    match pdata {
                        PropData::VecXY(v) => {
                            let endings = ["_X", "_Y"];
                            for inx in 0..2 {
                                let data = PropData::F32(v[inx]);
                                this_v.push((prop.name.to_string() + endings[inx], data));
                            }
                        }
                        PropData::VecXYZ(v) => {}
                        _ => {
                            this_v.push((prop.name.to_string(), pdata));
                        }
                    }
                }
            }
        }
        updated_vals
    }
}

pub fn get_current_steamid(
    tick: &i32,
    sid: &u64,
    userid_sid_map: &HashMap<u32, Vec<(u64, i32)>, RandomState>,
) -> u64 {
    match userid_sid_map.get(&(*sid as u32)) {
        None => {
            return 0;
        }
        Some(tups) => {
            if tups.len() == 1 {
                return tups[0].0;
            }
            for t in 0..tups.len() - 1 {
                if tups[t + 1].1 > *tick && tups[t].1 < *tick {
                    return tups[t].0;
                }
            }
            if tups[tups.len() - 1].1 < *tick {
                return tups[tups.len() - 1].0;
            }
            return tups[0].0;
        }
    };
}
pub fn get_current_entid(
    tick: &i32,
    sid: &u64,
    uid_eid_map: &HashMap<u32, Vec<(u32, i32)>, RandomState>,
) -> (u32, i32) {
    match uid_eid_map.get(&(*sid as u32)) {
        None => {
            //println!("no entid for {}", sid);
            return (0, -50000);
        }
        Some(tups) => {
            if tups.len() == 1 {
                return tups[0];
            }
            for t in 0..tups.len() - 1 {
                if tups[t + 1].1 > *tick && tups[t].1 < *tick {
                    return tups[t];
                }
            }
            if tups[tups.len() - 1].1 < *tick {
                return tups[tups.len() - 1];
            }
            return tups[0];
        }
    };
}

pub fn get_event_md(
    game_events: &Vec<GameEvent>,
    userid_sid_map: &HashMap<u32, Vec<(u64, i32)>, RandomState>,
    uid_eid_map: &HashMap<u32, Vec<(u32, i32)>, RandomState>,
) -> Vec<EventMd> {
    let mut md = vec![];
    for event in game_events {
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
            if f.name == "player_uid" {
                match f.data.as_ref().unwrap() {
                    KeyData::Uint64(x) => player = *x,
                    _ => {}
                }
            }
            if f.name == "attacker_uid" {
                match f.data.as_ref().unwrap() {
                    KeyData::Uint64(x) => attacker = Some(x),
                    _ => {}
                }
            }
        }
        let attacker_eid = match attacker {
            Some(sid) => Some(get_current_entid(&tick, sid, uid_eid_map)),
            None => None,
        };
        let player_eid = get_current_entid(&tick, &player, uid_eid_map);

        let attacker_sid = match attacker {
            Some(sid) => Some(get_current_steamid(&tick, sid, userid_sid_map)),
            None => None,
        };
        let player_sid = get_current_steamid(&tick, &player, userid_sid_map);

        md.push(EventMd {
            tick,
            player_eid: player_eid.0,
            player_sid: player_sid,
            attacker_eid: Some(attacker_eid.unwrap().0),
            attacker_sid: attacker_sid,
            player_min_tick: player_eid.1,
            attacker_min_tick: attacker_eid.unwrap().1,
        });
    }
    md
}
