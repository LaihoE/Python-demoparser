use super::parser::JobResult;
use super::parser::MsgBluePrint;
use crate::parsing::data_table::ServerClass;
use crate::parsing::data_table::ServerClasses;
use crate::parsing::parser::Parser;
use crate::parsing::read_bits::MyBitreader;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use ahash::HashMap;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use memmap2::Mmap;
use protobuf::Message;
use smallvec::SmallVec;
use std::convert::TryInto;
const SMALLVECSIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    pub props: HashMap<String, PropAtom>,
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
    sv_cls_map: &ServerClasses,
    wanted_props: &Vec<String>,
) -> JobResult {
    //return JobResult::None;
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
        sv_cls_map: &ServerClasses,
        tick: i32,
        wanted_props: &Vec<String>,
    ) -> Option<Vec<Vec<SingleEntOutput>>> {
        /*
        Main thing to understand here is that entity ids are
        sorted so we can break out early. Also first ~70 entids
        have predictable cls_ids, mainly entid < 64 = player.
        Higher entity ids are entities for other stuff,
        that are mostly not interesting.
        */

        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut all_props = vec![];
        let mut entity_id: i32 = -1;
        // println!("{:?}", n_upd_ents);
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

                match parse_ent_props(entity_id, &mut b, sv_cls_map, tick, wanted_props) {
                    Some(props) => {
                        all_props.push(props);
                    }
                    None => {}
                }
            } else {
                // IF ENTITY DOES EXIST
                match parse_ent_props(entity_id, &mut b, sv_cls_map, tick, wanted_props) {
                    Some(props) => {
                        all_props.push(props);
                    }
                    None => {}
                }
            }
        }
        return Some(all_props);
    }
}
#[inline(always)]
fn get_cls(ent_id: i32, sv_cls: &ServerClasses) -> &ServerClass {
    // Returns correct serverclass id based on entityid
    // This is the key to being able to go parallel across ticks
    match ent_id {
        275 => &sv_cls.world,
        _ => &sv_cls.player,
    }
}
#[inline(always)]
pub fn get_indicies(b: &mut MyBitreader) -> SmallVec<[i32; SMALLVECSIZE]> {
    /*
    Gets wanted prop indicies. The index maps to a Prop struct.
    Very hot function
    */
    let mut val = -1;
    let new_way = b.read_boolie().unwrap();
    let mut indicies: SmallVec<[_; SMALLVECSIZE]> = SmallVec::<[i32; SMALLVECSIZE]>::new();

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
pub fn skipvec(v: &SmallVec<[i32; SMALLVECSIZE]>) -> Option<i32> {
    let mut total_bits = 0;
    for item in v {
        let bits = T[*item as usize];
        if bits != 0 {
            total_bits += bits
        } else {
            return None;
        }
    }
    Some(total_bits)
}

#[inline(always)]
pub fn parse_ent_props(
    entity_id: i32,
    b: &mut MyBitreader,
    sv_cls_map: &ServerClasses,
    tick: i32,
    wanted_props: &Vec<String>,
) -> Option<Vec<SingleEntOutput>> {
    let sv_cls = get_cls(entity_id, sv_cls_map);
    let indicies = get_indicies(b);
    /*
    match skipvec(&indicies) {
        Some(bits) => {
            b.skip_many_bits(bits.try_into().unwrap());
            return None;
        }
        None => {}
    }
    */
    let mut props: Vec<SingleEntOutput> = Vec::with_capacity(2);

    for idx in &indicies {
        let prop = &sv_cls.props[*idx as usize];
        let pdata = b.decode(prop).unwrap();

        if wanted_props.contains(&prop.name) {
            match pdata {
                PropData::VecXY(xy) => {
                    let x = SingleEntOutput {
                        ent_id: entity_id,
                        prop_inx: 10000,
                        data: PropData::F32(xy[0]),
                    };
                    let y = SingleEntOutput {
                        ent_id: entity_id,
                        prop_inx: 10001,
                        data: PropData::F32(xy[1]),
                    };
                    props.push(x);
                    props.push(y);
                }
                _ => {
                    let data = SingleEntOutput {
                        ent_id: entity_id,
                        prop_inx: *idx,
                        data: pdata,
                    };
                    props.push(data);
                }
            }
        }
    }
    return Some(props);
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

const T: [i32; 2000] = [
    8, 32, 64, 32, 32, 32, 32, 64, 32, 0, 17, 0, 0, 0, 10, 11, 8, 32, 32, 32, 32, 32, 32, 0, 32, 0,
    8, 32, 4, 12, 0, 32, 0, 8, 0, 4, 12, 15, 32, 0, 8, 32, 4, 12, 15, 32, 0, 8, 0, 4, 12, 15, 32,
    0, 8, 0, 4, 12, 15, 32, 0, 8, 0, 4, 12, 15, 32, 0, 8, 0, 4, 12, 15, 32, 0, 8, 32, 0, 12, 0, 32,
    0, 8, 32, 0, 12, 15, 32, 0, 8, 0, 0, 12, 15, 32, 0, 8, 0, 0, 12, 0, 32, 0, 8, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 21, 21, 21, 21, 21, 21, 21, 21, 21, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 21, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    10, 10, 10, 10, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 32, 1, 1, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
    12, 0, 8, 1, 0, 0, 32, 32, 32, 32, 32, 32, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 32, 0, 0, 0, 0, 0,
    96, 32, 21, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 32, 32, 0, 0, 21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 4, 4,
    4, 4, 0, 0, 4, 0, 0, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 4, 4, 0, 0, 4, 0, 4, 0, 0, 0, 4,
    0, 0, 4, 4, 4, 4, 4, 4, 0, 4, 4, 0, 0, 0, 4, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 14, 0, 8, 0, 1, 8,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 1, 0,
    0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 8, 0, 8, 8, 8, 8, 8, 0, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 32, 0, 32, 0, 0, 96, 13, 0, 0, 15, 0, 6, 6, 0, 0,
    0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 8, 96, 0, 32, 0, 0,
    0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 21, 32, 3, 6, 0, 0, 21, 16, 3, 0, 0, 0, 12, 3, 0, 0, 0,
    0, 0, 0, 21, 0, 8, 32, 0, 21, 65, 0, 25, 0, 2, 32, 21, 21, 0, 0, 0, 0, 0, 8, 0, 0, 0, 14, 8, 8,
    0, 1, 0, 4, 16, 16, 8, 1, 1, 0, 1, 12, 4, 8, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 15,
    0, 0, 32, 32, 32, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 16, 0, 1, 0, 32, 32, 32, 5, 32, 21, 21, 0, 16,
    16, 16, 0, 0, 0, 0, 0, 0, 8, 8, 0, 1, 1, 32, 32, 32, 1, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 0, 0, 0, 0, 0, 32, 32,
    32, 32, 32, 32, 32, 32, 0, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 32, 32, 0, 32, 0, 0, 0,
    0, 0, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 32, 32, 0, 32, 32, 32, 32,
    32, 32, 0, 0, 0, 0, 0, 0, 0, 0, 32, 32, 32, 32, 32, 0, 0, 32, 32, 0, 0, 32, 0, 0, 32, 0, 0, 32,
    0, 32, 0, 0, 0, 0, 0, 0, 0, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 0, 0, 0, 0, 0, 0, 32, 0, 32, 32, 32, 0, 0, 32, 32, 32, 0, 0,
    32, 32, 32, 32, 32, 32, 0, 0, 0, 32, 0, 32, 0, 0, 0, 0, 0, 0, 32, 32, 32, 32, 0, 32, 32, 32,
    32, 32, 0, 32, 32, 32, 32, 32, 32, 32, 32, 0, 32, 32, 32, 32, 0, 0, 0, 0, 0, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 0, 0, 0,
    0, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 0, 0, 0, 0, 0, 0, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 0, 0, 0, 0, 0, 0, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 0, 0, 0, 0, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 0, 0, 0, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 0,
    0, 0, 0, 32, 32, 32, 32, 32, 32, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
