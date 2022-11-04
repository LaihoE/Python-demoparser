use crate::parsing::data_table::ServerClass;
use crate::parsing::read_bits::MyBitreader;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use crate::Demo;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;

use super::stringtables::UserInfo;

#[derive(Debug)]
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

impl Demo {
    pub fn parse_packet_entities(
        cls_map: &mut HashMap<u16, ServerClass, RandomState>,
        tick: i32,
        cls_bits: usize,
        pack_ents: CSVCMsg_PacketEntities,
        entities: &mut Vec<(u32, Entity)>,
        wanted_props: &Vec<String>,
        workhorse: &mut Vec<i32>,
        fp: i32,
        _highest_wanted_entid: i32,
        manager_id: &mut Option<u32>,
        rules_id: &mut Option<u32>,
        round: &mut i32,
        baselines: &HashMap<u32, HashMap<String, PropData>>,
    ) -> Option<Vec<u32>> {
        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut entity_id: i32 = -1;
        let mut player_ents = vec![];

        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var()? as i32);
            /*
            Disabled for now
            if entity_id > highest_wanted_entid {
                break;
            }
            */
            if b.read_boolie()? {
                b.read_boolie();
            } else if b.read_boolie()? {
                // IF ENTITY DOES NOT EXIST
                let cls_id = b.read_nbits(cls_bits.try_into().unwrap())?;
                let _ = b.read_nbits(10);

                let mut e = Entity {
                    class_id: cls_id,
                    entity_id: entity_id as u32,
                    props: HashMap::default(),
                };
                match baselines.get(&cls_id) {
                    Some(baseline) => {
                        for (k, v) in baseline {
                            if wanted_props.contains(k) {
                                let atom = PropAtom {
                                    prop_name: k.to_string(),
                                    data: v.clone(),
                                    tick: tick,
                                };
                                e.props.insert(k.to_string(), atom);
                            }
                        }
                    }
                    None => {}
                }

                if entity_id < 10000 {
                    match cls_map.get_mut(&(cls_id as u16)) {
                        Some(x) => {
                            if x.dt == "DT_CSPlayer" {
                                player_ents.push(entity_id as u32);
                            }
                            if cls_id == 39 {
                                *rules_id = Some(entity_id as u32);
                            }
                            if cls_id == 41 {
                                *manager_id = Some(entity_id as u32);
                                for p in &mut x.props {
                                    p.name = p.table.clone() + &p.name;
                                }
                            }
                        }
                        None => {}
                    }
                }
                update_entity(
                    &mut e,
                    &mut b,
                    cls_map,
                    wanted_props,
                    tick,
                    workhorse,
                    fp,
                    round,
                );
                entities[entity_id as usize] = (entity_id as u32, e);
            } else {
                // IF ENTITY DOES EXIST
                let ent = &mut entities[entity_id as usize];
                update_entity(
                    &mut ent.1,
                    &mut b,
                    cls_map,
                    wanted_props,
                    tick,
                    workhorse,
                    fp,
                    round,
                );
            }
        }
        if player_ents.len() > 0 {
            return Some(player_ents);
        } else {
        }
        None
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
    _fp: i32,
    round: &mut i32,
) -> Option<i32> {
    let mut val = -1;
    let new_way = b.read_boolie()?;
    let mut upd = 0;
    loop {
        val = b.read_inx(val, new_way)?;

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
        let pdata = b.decode(prop)?;

        //println!("INX: {}  e{}", inx, prop.name);
        // if prop is not wanted then dont create propdata from it
        if sv_cls.id != 39 && sv_cls.id != 41 && !is_wanted_prop_name(prop, &wanted_props) {
            continue;
        }

        match pdata {
            PropData::VecXY(v) => {
                let endings = ["_X", "_Y"];
                for inx in 0..2 {
                    let data = PropData::F32(v[inx]);
                    let name = prop.name.to_owned() + endings[inx];
                    let atom = PropAtom {
                        prop_name: name,
                        data: data,
                        tick: tick,
                    };
                    ent.props.insert(atom.prop_name.to_owned(), atom);
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
                        tick: tick,
                    };
                    ent.props.insert(atom.prop_name.to_owned(), atom);
                }
            }
            _ => {
                let atom = PropAtom {
                    prop_name: prop.name.to_string(),
                    data: pdata,
                    tick: tick,
                };

                if atom.prop_name == "m_totalRoundsPlayed" {
                    if let PropData::I32(r) = atom.data {
                        *round = r;
                    }
                }

                // Make sure player metadata isnt erased when players leave.
                if sv_cls.id == 41
                    || prop.name.contains("m_iCompetitiveRanking0")
                    || prop.name.contains("m_iTeam0")
                    || prop.name.contains("m_iCompetitiveWins0")
                    || prop.name.contains("m_iCompetitiveWins0")
                    || prop.name.contains("m_szCrosshairCodes0")
                {
                    match &atom.data {
                        PropData::I32(val) => {
                            if val != &0 {
                                ent.props.insert(prop.name.to_owned(), atom);
                            }
                        }
                        PropData::String(val) => {
                            if val.len() > 10 {
                                ent.props.insert(prop.name.to_owned(), atom);
                            }
                        }
                        _ => {}
                    }
                } else {
                    ent.props.insert(atom.prop_name.clone(), atom);
                }
            }
        }
    }
    // number of updated entries
    Some(upd.try_into().unwrap())
}
#[inline(always)]
pub fn update_entity(
    ent: &mut Entity,
    b: &mut MyBitreader,
    cls_map: &HashMap<u16, ServerClass, RandomState>,
    wanted_props: &Vec<String>,
    tick: i32,
    workhorse: &mut Vec<i32>,
    fp: i32,
    round: &mut i32,
) {
    let sv_cls = &cls_map[&(ent.class_id.try_into().unwrap())];
    parse_ent_props(ent, sv_cls, b, wanted_props, tick, workhorse, fp, round);
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
