use crate::parsing::data_table::ServerClass;
use crate::parsing::read_bits::BitReader;
//use crate::parsing::read_bits_skip::BitReader;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use crate::Demo;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use csgoproto::netmessages::CSVCMsg_SendTable;
use polars::export::num::Float;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    pub serial: u32,
    pub props: HashMap<String, PropAtom, RandomState>,
}

#[derive(Debug, Clone)]
pub struct Prop {
    pub name: String,
    //pub prop: Sendprop_t,
    pub arr: Option<Sendprop_t>,
    pub col: i32,
    pub data: Option<PropData>,
    pub flags: i32,
    pub num_elements: i32,
    pub num_bits: i32,
    pub low_value: f32,
    pub high_value: f32,
    pub priority: i32,
    pub p_type: i32,
}

#[inline(always)]
fn is_wanted_prop_name(this_prop: &Prop, wanted_props: &Vec<String>) -> bool {
    for prop in wanted_props {
        if prop == &this_prop.name {
            return true;
        }
    }
    false
}

impl Demo {
    pub fn parse_packet_entities(
        cls_map: &HashMap<u16, ServerClass, RandomState>,
        tick: i32,
        cls_bits: usize,
        pack_ents: CSVCMsg_PacketEntities,
        entities: &mut Vec<(u32, Entity)>,
        wanted_props: &Vec<String>,
        workhorse: &mut Vec<i32>,
    ) {
        let n_upd_ents = pack_ents.updated_entries();
        let left_over = (pack_ents.entity_data().len() % 4) as i32;
        let mut b = BitReader::new(pack_ents.entity_data(), left_over);
        b.read_uneven_end_bits();
        let mut entity_id: i32 = -1;

        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var() as i32);

            if b.read_bool() {
                b.read_bool();
            } else if b.read_bool() {
                // IF ENTITY DOES NOT EXIST

                let cls_id = b.read_nbits(cls_bits);
                let serial = b.read_nbits(10);

                let mut e = Entity {
                    class_id: cls_id,
                    entity_id: entity_id as u32,
                    serial: serial,
                    props: HashMap::default(),
                };
                update_entity(&mut e, &mut b, cls_map, wanted_props, tick, workhorse);
                entities[entity_id as usize] = (entity_id as u32, e);
            } else {
                // IF ENTITY DOES EXIST

                //let ent = entities.get_mut(&(entity_id.try_into().unwrap_or(99999999)));
                let ent = &mut entities[entity_id as usize];
                update_entity(&mut ent.1, &mut b, cls_map, wanted_props, tick, workhorse);
            }
        }
    }
}
#[inline(always)]
pub fn parse_ent_props(
    ent: &mut Entity,
    sv_cls: &ServerClass,
    b: &mut BitReader<&[u8]>,
    wanted_props: &Vec<String>,
    tick: i32,
    workhorse: &mut Vec<i32>,
) {
    let mut val = -1;
    let new_way = b.read_bool();
    let mut upd = 0;
    loop {
        val = b.read_inx(val, new_way);
        if val == -1 {
            break;
        }
        // Reuse same vec to avoid alloc vec every time
        workhorse[upd] = val;
        upd += 1;
    }

    for i in 0..upd {
        let inx = workhorse[i];
        let prop = &sv_cls.props[inx as usize];
        let pdata = b.decode(prop);

        if !is_wanted_prop_name(prop, &wanted_props) {
            continue;
        }

        match pdata {
            PropData::VecXY(v) => {
                let endings = ["_X", "_Y"];
                for inx in 0..2 {
                    let data = PropData::F32(v[inx]);
                    let name = prop.name.to_string() + endings[inx];
                    let atom = PropAtom {
                        prop_name: name,
                        data: data,
                        tick: tick,
                    };
                    ent.props.insert(atom.prop_name.clone(), atom);
                }
            }
            PropData::VecXYZ(v) => {
                let endings = ["_X", "_Y", "_Z"];
                for inx in 0..3 {
                    let data = PropData::F32(v[inx]);
                    let name = prop.name.to_string() + endings[inx];
                    let atom = PropAtom {
                        prop_name: name,
                        data: data,
                        tick: tick,
                    };
                    ent.props.insert(atom.prop_name.clone(), atom);
                }
            }
            _ => {
                let atom = PropAtom {
                    prop_name: prop.name.to_string(),
                    data: pdata,
                    tick: tick,
                };
                ent.props.insert(atom.prop_name.clone(), atom);
            }
        }
    }
}
#[inline(always)]
pub fn update_entity(
    ent: &mut Entity,
    b: &mut BitReader<&[u8]>,
    cls_map: &HashMap<u16, ServerClass, RandomState>,
    wanted_props: &Vec<String>,
    tick: i32,
    workhorse: &mut Vec<i32>,
) {
    let sv_cls = &cls_map[&(ent.class_id.try_into().unwrap())];
    parse_ent_props(ent, sv_cls, b, wanted_props, tick, workhorse);
}
