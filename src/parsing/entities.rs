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
#[derive(Debug, Clone)]
pub struct PacketEntsOutput {
    pub data: Vec<Vec<SingleEntOutput>>,
    pub tick: i32,
}
#[derive(Debug, Clone)]
pub struct SingleEntOutput {
    pub ent_id: i32,
    pub prop_inx: i32,
    pub data: PropData,
}

pub fn parse_packet_entities(
    blueprint: &MsgBluePrint,
    mmap: &Mmap,
    sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
    wanted_props: &Vec<String>,
) -> JobResult {
    let wanted_bytes = &mmap[blueprint.start_idx..blueprint.end_idx];
    let msg = Message::parse_from_bytes(wanted_bytes).unwrap();
    let outputs = Parser::_parse_packet_entities(msg, sv_cls_map, blueprint.tick, wanted_props);
    match outputs {
        Some(output) => JobResult::PacketEntities(Some(PacketEntsOutput {
            data: output,
            tick: blueprint.tick,
        })),
        None => JobResult::None,
    }
}

impl Parser {
    pub fn _parse_packet_entities(
        pack_ents: CSVCMsg_PacketEntities,
        sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
        tick: i32,
        wanted_props: &Vec<String>,
    ) -> Option<Vec<Vec<SingleEntOutput>>> {
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
                let this_ent_props =
                    parse_ent_props(entity_id, &mut b, sv_cls_map, tick, wanted_props);
                if this_ent_props.as_ref().unwrap().len() > 0 {
                    all_props.extend(this_ent_props);
                }
            } else {
                // IF ENTITY DOES EXIST
                let this_ent_props =
                    parse_ent_props(entity_id, &mut b, sv_cls_map, tick, wanted_props);
                if this_ent_props.as_ref().unwrap().len() > 0 {
                    all_props.extend(this_ent_props);
                }
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
    wanted_props: &Vec<String>,
) -> Option<Vec<SingleEntOutput>> {
    let cls_id = get_cls_id(entity_id);
    let m = sv_cls_map;
    let sv_cls = m.get(&cls_id).unwrap();
    let indicies = get_indicies(b);
    let mut props: Vec<SingleEntOutput> = vec![];
    for idx in &indicies {
        let prop = &sv_cls.props[*idx as usize];

        let pdata = b.decode(prop).unwrap();
        if wanted_props.contains(&prop.name) {
            if prop.name.len() > 4 && tick == 69 {
                let output = SingleEntOutput {
                    ent_id: entity_id,
                    prop_inx: *idx,
                    data: pdata,
                };
                props.push(output);
            }
        }
    }
    let after = b.reader.bits_remaining();
    /*
        println!(
            "bits {} p:{} {:?}",
            before.unwrap() - after.unwrap(),
            p,
            indicies
        );
    */
    Some(props)
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
