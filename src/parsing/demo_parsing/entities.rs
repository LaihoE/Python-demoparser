use super::stringtables::UserInfo;
use crate::parsing::demo_parsing::*;
use crate::parsing::parser::JobResult;
use crate::parsing::parser::MsgBluePrint;
use crate::parsing::parser::Parser;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use ahash::RandomState;
use bitter::BitReader;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use memmap2::Mmap;
use protobuf::Message;
use serde::Deserialize;
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
#[derive(Debug, Clone, Deserialize)]
pub struct SingleEntOutput {
    pub ent_id: i32,
    pub prop_inx: i32,
    pub data: PropData,
}
#[derive(Debug, Deserialize, Clone)]
pub struct PacketEntsOutput {
    pub data: Vec<SingleEntOutput>,
    pub tick: i32,
    pub byte: usize,
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
impl Parser {
    pub fn parse_packet_entities(
        blueprint: &MsgBluePrint,
        mmap: &Mmap,
        sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
    ) -> JobResult {
        let wanted_bytes = &mmap[blueprint.start_idx..blueprint.end_idx];
        let msg = Message::parse_from_bytes(wanted_bytes).unwrap();
        let result = Parser::_parse_packet_entities(msg, sv_cls_map, blueprint.tick);
        match result {
            None => JobResult::None,
            Some(p) => JobResult::PacketEntities(PacketEntsOutput {
                data: p,
                tick: blueprint.tick,
                byte: blueprint.byte,
            }),
        }
    }

    pub fn _parse_packet_entities(
        pack_ents: CSVCMsg_PacketEntities,
        sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
        tick: i32,
    ) -> Option<Vec<SingleEntOutput>> {
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

            if entity_id > 71 {
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
    // Returns correct serverclass id based on entity id
    // This is the key to being able to go parallel across ticks
    assert!(ent_id < 72);

    match ent_id {
        // WORLD
        0 => 275,
        // TEAM
        65 => 43,
        66 => 43,
        67 => 43,
        68 => 43,
        69 => 43,
        // MANAGER
        70 => 41,
        // RULES
        71 => 39,
        _ => 40,
    }
}
#[inline(always)]
pub fn parse_indicies(b: &mut MyBitreader) -> SmallVec<[i32; 64]> {
    /*
    Gets wanted prop indicies. The index maps to a Prop struct.
    For example Player serverclass (id=40) with index 20 gives m_angEyeAngles[0]
    */
    let mut val = -1;
    let before = b.reader.bits_remaining().unwrap();
    let p = b.reader.peek(54);
    let new_way = b.read_boolie().unwrap();
    let mut indicies: SmallVec<[_; 64]> = SmallVec::<[i32; 64]>::new();

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
) -> Vec<SingleEntOutput> {
    let cls_id = get_cls_id(entity_id);
    let svc_map = sv_cls_map;
    let sv_cls = svc_map.get(&cls_id).unwrap();
    let indicies = parse_indicies(b);

    let mut props: Vec<SingleEntOutput> = vec![];

    for idx in indicies {
        if idx as usize > sv_cls.props.len() {
            println!(">>> {}", entity_id);
        }
        let prop = &sv_cls.props[idx as usize];
        let pdata = b.decode(prop).unwrap();
        match pdata {
            PropData::VecXY(v) => {
                // Extract vec into their own props
                props.push(SingleEntOutput {
                    ent_id: entity_id,
                    prop_inx: 10000,
                    data: PropData::F32(v[0]),
                });
                props.push(SingleEntOutput {
                    ent_id: entity_id,
                    prop_inx: 10001,
                    data: PropData::F32(v[1]),
                });
            }
            _ => {
                let data = SingleEntOutput {
                    ent_id: entity_id,
                    prop_inx: idx,
                    data: pdata,
                };
                props.push(data);
            }
        }
    }
    props
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
