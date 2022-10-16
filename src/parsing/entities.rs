use crate::parsing::data_table::ServerClass;
//use crate::parsing::read_bits::BitReader;
use crate::parsing::read_bits_skip::MyBitreader;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use crate::Demo;
use ahash::RandomState;
use bitter::{BitReader, LittleEndianReader};
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use csgoproto::netmessages::CSVCMsg_SendTable;
use polars::export::num::Float;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::time::Instant;

#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
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
        bit_size: &mut HashMap<u32, HashSet<u16>, RandomState>,
        insert: bool,
        bit_lookup: &HashMap<u32, u16, RandomState>,
    ) {
        let n_upd_ents = pack_ents.updated_entries();
        //println!("{:?}", pack_ents.entity_data());
        let mut b = MyBitreader::new(pack_ents.entity_data());

        //panic!("YEE");
        //b.read_uneven_end_bits();
        let mut entity_id: i32 = -1;

        for _ in 0..n_upd_ents {
            //println!("Y");

            entity_id += 1 + (b.read_u_bit_var() as i32);

            //println!("{}", pack_ents.entity_data().len());
            /*
            for i in 0..50 {
                println!("{} {i}", b.read_boolie());
            }
            panic!();
            */
            if b.read_boolie() {
                b.read_boolie();
            } else if b.read_boolie() {
                // IF ENTITY DOES NOT EXIST

                let cls_id = b.read_nbits(cls_bits.try_into().unwrap());

                let x = b.read_nbits(10);

                //println!("cls_id: {}", cls_id);
                //println!("SERIAL: {} {}, {:?}", entity_id, x, cls_map[&(cls_id as u16)].dt);

                let mut e = Entity {
                    class_id: cls_id,
                    entity_id: entity_id as u32,
                    props: HashMap::default(),
                };
                update_entity(
                    &mut e,
                    &mut b,
                    cls_map,
                    wanted_props,
                    tick,
                    workhorse,
                    bit_size,
                    insert,
                    bit_lookup,
                );
                entities[entity_id as usize] = (entity_id as u32, e);
            } else {
                // IF ENTITY DOES EXIST
                //let ent = entities.get_mut(&(entity_id.try_into().unwrap_or(99999999)));
                let ent = &mut entities[entity_id as usize];
                update_entity(
                    &mut ent.1,
                    &mut b,
                    cls_map,
                    wanted_props,
                    tick,
                    workhorse,
                    bit_size,
                    insert,
                    bit_lookup,
                );
            }
        }
    }
}
#[inline(always)]
pub fn parse_ent_props(
    ent: &mut Entity,
    sv_cls: &ServerClass,
    b: &mut MyBitreader,
    wanted_props: &Vec<String>,
    tick: i32,
    workhorse: &mut Vec<i32>,
    bit_size: &mut HashMap<u32, HashSet<u16>, RandomState>,
    insert: bool,
    bit_lookup: &HashMap<u32, u16, RandomState>,
) {
    let mut val = -1;
    let new_way = b.read_boolie();
    let mut upd = 0;
    loop {
        val = b.read_inx(val, new_way);

        if val == -1 {
            break;
        }
        // panic!("k");
        // Reuse same vec to avoid alloc vec every time

        workhorse[upd] = val;
        upd += 1;
    }
    //println!("{:?}", workhorse);
    for i in 0..upd {
        let inx = workhorse[i];
        let prop = &sv_cls.props[inx as usize];
        // Need to combine two integers into a key.
        // Cast both to bigger and left shift into combined
        // sv_cls.id = 000000123456
        // inx       = 000000777777
        // k         = 123456777777
        let tt: u32 = 16;
        let k = (sv_cls.id as u32) << tt | (inx as u32);
        let mut pos_before = 0;
        pos_before = if insert {
            b.reader.bits_remaining().unwrap()
        } else {
            0
        };
        if bit_lookup.contains_key(&k) && insert == false {
            let elem = &bit_lookup[&k];
            //println!("{} {} {}", prop.name, elem, sv_cls.dt);
            if elem <= &56 {
                b.reader.refill_lookahead();
                //println!("refilled: {}", x);
                b.reader.consume(*elem as u32);
            } else if elem == &64 {
                b.reader.refill_lookahead();
                //println!("refilled: {}", x);
                b.reader.consume(54);
                b.reader.refill_lookahead();
                //println!("refilled: {}", x);
                b.reader.consume(10);
            } else if elem == &96 {
                b.reader.refill_lookahead();
                //println!("refilled: {}", x);
                b.reader.consume(54);
                b.reader.refill_lookahead();
                //println!("refilled: {}", x);
                b.reader.consume(42);
            } else {
                let pdata = b.decode(prop);
            }
        } else {
            let pdata = b.decode(prop);
        }

        //let pdata = b.decode(prop);

        //let k = sv_cls.dt.to_string() + &prop.name.to_string();
        //let before = b.reader.bits_remaining().unwrap();
        if insert {
            let pos_after = b.reader.bits_remaining().unwrap();

            let v = pos_before - pos_after;
            /*
            if prop.flags == 8 {
                println!(
                    "{} {} {} {}",
                    v,
                    prop.num_bits,
                    prop.flags,
                    prop.flags & (1 << 5)
                );
            }
            */
            /*
            if v == prop.num_bits as usize {
                println!(
                    "{} {} {} {}",
                    v,
                    prop.num_bits,
                    prop.flags,
                    prop.flags & (1 << 5)
                );
            }
            */
            bit_size
                .entry(k)
                .or_insert(HashSet::new())
                .insert(v.try_into().unwrap());
        }

        //let k = sv_cls.id;
        //let v = inx;
        //println!("{} {}", k, v);

        //bit_size.insert(k, v as u32);
        /*
        if prop.name != "m_vecOrigin" {
            println!(
                "{:?} {} {} {} {:?} DIF: {} t{}",
                sv_cls.dt,
                prop.name,
                prop.flags,
                prop.num_bits,
                prop.p_type,
                pos_before - pos_after,
                tick,
            );
        }
        */
        /*
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
        */
    }
}
#[inline(always)]
pub fn update_entity(
    ent: &mut Entity,
    b: &mut MyBitreader,
    cls_map: &HashMap<u16, ServerClass, RandomState>,
    wanted_props: &Vec<String>,
    tick: i32,
    workhorse: &mut Vec<i32>,
    bit_size: &mut HashMap<u32, HashSet<u16>, RandomState>,
    insert: bool,
    bit_lookup: &HashMap<u32, u16, RandomState>,
) {
    let sv_cls = &cls_map[&(ent.class_id.try_into().unwrap())];
    parse_ent_props(
        ent,
        sv_cls,
        b,
        wanted_props,
        tick,
        workhorse,
        bit_size,
        insert,
        bit_lookup,
    );
}
