use super::parser::Maps;
use super::parser::MsgBluePrint;
use super::parser::ParserSettings;
use super::parser::ParserState;
use super::stringtables::UserInfo;
use crate::parsing::data_table::ServerClass;
use crate::parsing::parser::Parser;
use crate::parsing::read_bits::MyBitreader;
use crate::parsing::variants::create_default_from_pdata;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use crate::VarVec;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use dashmap::DashMap;
use memmap2::Mmap;
use protobuf::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    pub props: HashMap<String, PropAtom, RandomState>,
}

#[derive(Debug, Clone)]
pub struct Prop {
    pub name: String,
    pub table: String,
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

pub fn parse_packet_entities(
    blueprint: MsgBluePrint,
    mmap: Arc<Mmap>,
    sv_cls_map: Arc<RwLock<HashMap<u16, ServerClass, RandomState>>>,
    data: Arc<DashMap<String, HashMap<u32, VarVec>>>,
) {
    let wanted_bytes = &mmap[blueprint.start_idx..blueprint.end_inx];
    let msg = Message::parse_from_bytes(wanted_bytes).unwrap();
    Parser::_parse_packet_entities(msg, sv_cls_map, data, blueprint.tick);
}

impl Parser {
    pub fn _parse_packet_entities(
        pack_ents: CSVCMsg_PacketEntities,
        sv_cls_map: Arc<RwLock<HashMap<u16, ServerClass, RandomState>>>,
        data: Arc<DashMap<String, HashMap<u32, VarVec>>>,
        tick: i32,
    ) -> Option<Vec<u32>> {
        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut entity_id: i32 = -1;
        let mut player_ents = vec![];

        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var()? as i32);
            if entity_id > 64 {
                break;
            }
            if b.read_boolie()? {
                // Checks for if entity should be destroyed, don't see this being useful
                b.read_boolie();
            } else if b.read_boolie()? {
                // IF ENTITY DOES NOT EXIST
                let cls_id = b.read_nbits(9)?;
                let _ = b.read_nbits(10);
                let mut e = Entity {
                    class_id: cls_id,
                    entity_id: entity_id as u32,
                    props: HashMap::default(),
                };
                parse_ent_props(entity_id, &mut b, sv_cls_map.clone(), data.clone(), tick);
            } else {
                // IF ENTITY DOES EXIST
                parse_ent_props(entity_id, &mut b, sv_cls_map.clone(), data.clone(), tick);
            }
        }
        if player_ents.len() > 0 {
            return Some(player_ents);
        } else {
        }
        None
    }
}

fn get_cls_id(ent_id: i32) -> u16 {
    assert!(ent_id < 65);
    match ent_id {
        0 => 275,
        _ => 40,
    }
}

#[inline(always)]
pub fn parse_ent_props(
    entity_id: i32,
    b: &mut MyBitreader,
    sv_cls_map: Arc<RwLock<HashMap<u16, ServerClass, RandomState>>>,
    data: Arc<DashMap<String, HashMap<u32, VarVec>>>,
    tick: i32,
) -> Option<i32> {
    let mut val = -1;
    let new_way = b.read_boolie()?;
    let mut upd = 0;
    //let ent = &mut state.entities[ent_id as usize].1;
    //let cls_id = ent.class_id;
    let cls_id = get_cls_id(entity_id);
    let m = sv_cls_map.read().unwrap();
    let sv_cls = m.get(&cls_id).unwrap();
    let mut indicies = vec![];
    loop {
        val = b.read_inx(val, new_way).unwrap();
        if val == -1 {
            break;
        }
        indicies.push(val);
    }
    let mut props = vec![];
    for idx in indicies {
        let prop = &sv_cls.props[idx as usize];
        let pdata = b.decode(prop).unwrap();
        //let wanted_props = vec!["m_iHealth".to_string()];

        if prop.name != "m_iHealth" {
            continue;
        }
        //if sv_cls.id != 39 && sv_cls.id != 41 && !is_wanted_prop_name(prop, &wanted_props) {
        //continue;
        //}
        //println!("{}", prop.name);
        match pdata {
            PropData::VecXY(v) => {
                let endings = ["_X", "_Y"];
                for inx in 0..2 {
                    let data = PropData::F32(v[inx]);

                    let name = prop.name.to_owned() + endings[inx];
                    let atom = PropAtom {
                        prop_name: name,
                        data: data,
                        tick: 22, //tick: state.tick,
                    };
                    //ent.props.insert(atom.prop_name.to_owned(), atom);
                }
            }
            PropData::VecXYZ(v) => {
                let endings = ["_X", "_Y", "_Z"];
                for inx in 0..3 {
                    let data = PropData::F32(v[inx]);
                    let name = prop.name.to_owned() + endings[inx];
                    let atom = PropAtom {
                        prop_name: name,
                        data: data,
                        tick: 22, //tick: state.tick,
                    };
                    //ent.props.insert(atom.prop_name.to_owned(), atom);
                }
            }
            _ => {
                if prop.name.len() > 4 {
                    props.push((prop.name.to_owned(), pdata));
                }
            }
        }
    }
    /*
    for prop in props {
        match data.get_mut(&prop.0) {
            Some(mut inner) => match inner.get_mut(&(entity_id as u32)) {
                Some(vv) => {
                    vv.insert_propdata((tick / 2) as usize, prop.1);
                }
                None => {
                    inner.insert(entity_id as u32, create_default_from_pdata(&prop.1, 200000));
                    inner
                        .get_mut(&(entity_id as u32))
                        .unwrap()
                        .insert_propdata((tick / 2) as usize, prop.1);
                }
            },
            None => {
                data.insert(prop.0.clone(), HashMap::default());
            }
        }
    }
    */
    // number of updated entries
    Some(upd.try_into().unwrap())
}

#[inline(always)]
pub fn highest_wanted_entid(
    entids_not_connected: &HashSet<u32>,
    players: &HashMap<u64, UserInfo, RandomState>,
    wanted_players: &Vec<u64>,
) -> i32 {
    /*
    Returns highest wanted entity_id to be able to
    early exit parsing packet entites after all our
    wanted players are parsed
    */
    let mut highest_wanted = 0;
    for player in players {
        if wanted_players.contains(&player.0) {
            let wanted_ent_id = player.1.entity_id;
            if highest_wanted < wanted_ent_id {
                highest_wanted = wanted_ent_id;
            }
            for eid in 1..wanted_ent_id {
                if entids_not_connected.contains(&eid) {
                    return 999999;
                }
            }
        }
    }
    if highest_wanted > 0 {
        return highest_wanted as i32;
    } else {
        return 999999;
    }
}

pub fn parse_baselines(
    data: &[u8],
    sv_cls: &ServerClass,
    baselines: &mut HashMap<u32, HashMap<String, PropData>>,
) {
    let mut b = MyBitreader::new(data);
    let mut val = -1;
    let new_way = b.read_boolie().unwrap();
    let mut indicies = vec![];
    let mut baseline: HashMap<String, PropData> = HashMap::default();
    loop {
        val = b.read_inx(val, new_way).unwrap();
        if val == -1 {
            break;
        }
        indicies.push(val);
    }
    for inx in indicies {
        let prop = &sv_cls.props[inx as usize];
        let pdata = b.decode(prop).unwrap();
        baseline.insert(prop.name.to_owned(), pdata);
    }
    baselines.insert(sv_cls.id.try_into().unwrap(), baseline);
}

#[inline(always)]
fn is_wanted_prop_name(this_prop: &Prop, wanted_props: &Vec<String>) -> bool {
    for prop in wanted_props {
        if prop == &this_prop.name
            || this_prop.name == "m_hActiveWeapon"
            || this_prop.name == "m_iClip1"
            || this_prop.name == "m_iItemDefinitionIndex"
        {
            return true;
        }
    }
    false
}
