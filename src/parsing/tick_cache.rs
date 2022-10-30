use super::{
    data_table::ServerClass,
    entities,
    variants::create_default_from_pdata,
    variants::{PropData, VarVec},
};
use crate::parsing::entities::Entity;
use crate::parsing::read_bits::MyBitreader;
use ahash::RandomState;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use phf::phf_map;
use protobuf::Message;
use std::collections::HashMap;

pub struct TickCache {
    ticks: Vec<(usize, usize)>,
    pub ents: HashMap<u32, HashMap<u32, VarVec>>,
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
    pub fn get_prop_at_tick(&self, tick: i32, prop_inx: u32, ent_id: u32) -> Option<PropData> {
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
    pub fn insert_cache(&mut self, tick: i32, prop_inx: u32, prop: PropData, ent_id: u32) {
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
                    e.insert(prop_inx, create_default_from_pdata(prop, 100000));
                    // Bug watch out
                    // v.insert_propdata(tick as usize, prop);
                }
            },
            None => {
                self.ents.insert(ent_id, HashMap::default());
            }
        }
    }
    pub fn insert_cache_multiple(&mut self, tick: i32, hm: &HashMap<u32, Vec<(u32, PropData)>>) {
        for (player, data) in hm {
            for (prop_inx, p) in data {
                self.insert_cache(tick, *prop_inx, p.clone(), *player)
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

    // Stripped down version of the main function
    pub fn parse_packet_ents_simple(
        &mut self,
        pack_ents: CSVCMsg_PacketEntities,
        entities: &Vec<(u32, Entity)>,
        serverclass_map: &HashMap<u16, ServerClass, RandomState>,
    ) -> HashMap<u32, Vec<(u32, PropData)>> {
        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut entity_id: i32 = -1;

        let mut updated_vals: HashMap<u32, Vec<(u32, PropData)>> = HashMap::default();

        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var().unwrap() as i32);
            if entity_id > 20 {
                break;
            }
            //println!("{}", entity_id);

            if b.read_boolie().unwrap() {
                b.read_boolie().unwrap();
            } else if b.read_boolie().unwrap() {
                panic!("Tried to create new ent in speedy mode {}", entity_id);
            } else {
                // IF ENTITY DOES EXIST
                let ent = &entities[entity_id as usize];
                let sv_cls = &serverclass_map[&(ent.1.class_id as u16)];
                if sv_cls.dt != "DT_CSPlayer" {
                    println!("NOT PLAYER: {}", entity_id);
                    break;
                }
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
                            //let endings = ["_X", "_Y"];
                            for inx in 0..2 {
                                let data = PropData::F32(v[inx]);
                                this_v.push(((10000 + inx) as u32, data));
                            }
                        }
                        PropData::VecXYZ(v) => {}
                        _ => {
                            this_v.push((inx.try_into().unwrap(), pdata));
                        }
                    }
                }
            }
        }
        updated_vals
    }
}
