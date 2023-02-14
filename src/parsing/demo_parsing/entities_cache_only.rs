use crate::parsing::demo_parsing::*;
use crate::parsing::parser::JobResult;
use crate::parsing::parser::MsgBluePrint;
use crate::parsing::parser::Parser;
use crate::parsing::variants::PropData;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use memmap2::Mmap;
use protobuf::Message;
use smallvec::SmallVec;
use std::collections::HashMap;

/*
Stripped down version of entities.rs where this one gets prop indicies
as fast as possible. These are what are written into the cache.
*/

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

#[derive(Debug)]
pub struct EntityIndicies {
    pub byte: u64,
    pub entid: i32,
    pub tick: i32,
    pub prop_indicies: SmallVec<[i32; 32]>,
}

impl Parser {
    pub fn parse_packet_entities_indicies(
        blueprint: &MsgBluePrint,
        mmap: &Mmap,
        sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
    ) -> JobResult {
        // Demo is corrupt
        if blueprint.end_idx > mmap.len() {
            return JobResult::None;
        }
        let wanted_bytes = &mmap[blueprint.start_idx..blueprint.end_idx];
        let msg = Message::parse_from_bytes(wanted_bytes).unwrap();
        let result = Parser::_parse_packet_entities_indicies(
            msg,
            sv_cls_map,
            blueprint.tick,
            blueprint.byte as u64,
        );
        JobResult::PacketEntitiesIndicies(result)
    }

    fn _parse_packet_entities_indicies(
        pack_ents: CSVCMsg_PacketEntities,
        sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
        tick: i32,
        byte: u64,
    ) -> Vec<EntityIndicies> {
        /*
        Main thing to understand here is that entity ids are
        sorted so we can break out early. Also first ~70 entids
        have predictable cls_ids, mainly entid < 64 = player.
        Higher entity ids are entities for other props etc.
        that are mostly not interesting.
        */
        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut outputs = Vec::with_capacity(12);
        let mut entity_id: i32 = -1;
        for _ in 0..n_upd_ents {
            entity_id += match b.read_u_bit_var() {
                Some(eid_plus) => eid_plus as i32 + 1,
                None => break,
            };

            if entity_id > 71 {
                break;
            }
            if b.read_boolie().unwrap() {
                // Checks if entity should be destroyed, don't see this being useful for parser
                b.read_boolie();
            } else if b.read_boolie().unwrap() {
                // IF ENTITY DOES NOT EXIST
                // These bits are for creating the ent but we use hack for it so not needed
                let _ = b.read_nbits(19).unwrap();
                let indicies = parse_ent_props_indicies(entity_id, &mut b, sv_cls_map, tick);
                match indicies {
                    Some(idc) => {
                        outputs.push(EntityIndicies {
                            byte: byte,
                            tick: tick,
                            entid: entity_id,
                            prop_indicies: idc,
                        });
                    }
                    None => break,
                }
            } else {
                // IF ENTITY DOES EXIST
                let indicies = parse_ent_props_indicies(entity_id, &mut b, sv_cls_map, tick);
                match indicies {
                    Some(idc) => {
                        outputs.push(EntityIndicies {
                            byte: byte,
                            tick: tick,
                            entid: entity_id,
                            prop_indicies: idc,
                        });
                    }
                    None => break,
                }
            }
        }

        return outputs;
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

fn parse_indicies(b: &mut MyBitreader) -> Option<SmallVec<[i32; 32]>> {
    /*
    Gets wanted prop indicies. The index maps to a Prop struct.
    For example Player serverclass (id=40) with index 20 gives m_angEyeAngles[0]
    */
    let mut val = -1;
    let new_way = b.read_boolie()?;
    let mut indicies: SmallVec<[_; 32]> = SmallVec::<[i32; 32]>::new();
    loop {
        val = b.read_inx(val, new_way)?;
        if val == -1 {
            break;
        }
        indicies.push(val);
    }
    Some(indicies)
}

fn parse_ent_props_indicies(
    entity_id: i32,
    b: &mut MyBitreader,
    sv_cls_map: &HashMap<u16, ServerClass, RandomState>,
    tick: i32,
) -> Option<SmallVec<[i32; 32]>> {
    let cls_id = get_cls_id(entity_id);
    let m = sv_cls_map;

    let sv_cls = match m.get(&cls_id) {
        Some(svc) => svc,
        None => {
            return None;
        }
    };
    let indicies = match parse_indicies(b) {
        Some(idc) => idc,
        None => return None,
    };

    for idx in &indicies {
        match sv_cls.props.get(*idx as usize) {
            Some(x) => {
                let _ = b.decode(x);
            }
            None => return None,
        }
    }
    Some(indicies)
}
