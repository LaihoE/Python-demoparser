use super::parser::JobResult;
use super::parser::MsgBluePrint;
use super::stringtables::UserInfo;
use crate::parsing::data_table::ServerClass;
use crate::parsing::parser::Parser;
use crate::parsing::read_bits::MyBitreader;
use crate::parsing::variants::create_default_from_pdata;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use crate::VarVec;
use ahash::RandomState;
use bitter::BitReader;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use memmap2::Mmap;
use protobuf::Message;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;

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
    blueprint: &MsgBluePrint,
    mmap: &Mmap,
    sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
) -> JobResult {
    let wanted_bytes = &mmap[blueprint.start_idx..blueprint.end_idx];
    let msg = Message::parse_from_bytes(wanted_bytes).unwrap();
    JobResult::PacketEntities(Parser::_parse_packet_entities(
        msg,
        sv_cls_map,
        blueprint.tick,
    ))
}

impl Parser {
    pub fn _parse_packet_entities(
        pack_ents: CSVCMsg_PacketEntities,
        sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
        tick: i32,
    ) -> Option<Vec<SmallVec<[(i32, PropData); 1]>>> {
        /*
        Main thing to understand here is that entity ids are
        sorted so we can break out early. Also first ~70 entids
        have predictable cls_ids, mainly entid < 64 = player.
        Higher entity ids are entities for other props etc.
        that are mostly not interesting.
        */
        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut all_props = vec![];
        let mut entity_id: i32 = -1;
        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var()? as i32);
            if entity_id > 64 {
                break;
            }
            if b.read_boolie()? {
                // Checks if entity should be destroyed, don't see this being useful for parser
                b.read_boolie();
            } else if b.read_boolie()? {
                // IF ENTITY DOES NOT EXIST
                // These bits are for creating the ent but we use hack for it so not needed
                let _ = b.read_nbits(19)?;
                all_props.extend(parse_ent_props(entity_id, &mut b, sv_cls_map, tick));
            } else {
                // IF ENTITY DOES EXIST
                all_props.extend(parse_ent_props(entity_id, &mut b, sv_cls_map, tick));
            }
        }
        return Some(all_props);
    }
}
#[inline(always)]
fn get_cls_id(ent_id: i32) -> u16 {
    // Returns correct serverclass id based on entityid
    // This is the key to being able to go parallel across ticks
    assert!(ent_id < 65);
    match ent_id {
        0 => 275,
        _ => 40,
    }
}
#[inline(always)]
pub fn get_indicies(b: &mut MyBitreader) -> SmallVec<[i32; 128]> {
    /*
    Gets wanted prop indicies. The index maps to a Prop struct.
    For example Player serverclass (id=40) with index 20 gives m_angEyeAngles[0]
    */
    let mut val = -1;
    let new_way = b.read_boolie().unwrap();
    let mut indicies: SmallVec<[_; 128]> = SmallVec::<[i32; 128]>::new();
    loop {
        val = b.read_inx(val, new_way).unwrap();
        if val == -1 {
            break;
        }
        indicies.push(val);
    }
    indicies
}
#[inline(always)]
pub fn parse_ent_props(
    entity_id: i32,
    b: &mut MyBitreader,
    sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
    tick: i32,
) -> Option<SmallVec<[(i32, PropData); 1]>> {
    let cls_id = get_cls_id(entity_id);
    let m = sv_cls_map;
    let sv_cls = m.get(&cls_id).unwrap();
    let indicies = get_indicies(b);
    let mut props: SmallVec<[(i32, PropData); 1]> = SmallVec::<[(i32, PropData); 1]>::new();
    for idx in indicies {
        let prop = &sv_cls.props[idx as usize];
        let pdata = b.decode(prop).unwrap();
        if prop.name != "m_angEyeAngles[0]" || tick != 6999 {
            continue;
        }
        if prop.name.len() > 4 {
            props.push((idx, pdata));
        }
    }
    Some(props)
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
