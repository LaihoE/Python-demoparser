use super::ByteReader;
use crate::parsing::demo_parsing::data_table::ServerClass;
use crate::parsing::demo_parsing::read_bits::MyBitreader;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use crate::Parser;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use protobuf::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;

//use super::stringtables::UserInfo;

#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    //pub props: HashMap<String, PropAtom, RandomState>,
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
    pub fn parse_packet_entities(&mut self, byte_reader: &mut ByteReader, size: usize) {
        let wanted_bytes = &self.bytes[byte_reader.byte_idx..byte_reader.byte_idx + size as usize];
        byte_reader.skip_n_bytes(size.try_into().unwrap());
        let pack_ents: CSVCMsg_PacketEntities = Message::parse_from_bytes(wanted_bytes).unwrap();

        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut entity_id: i32 = -1;

        //self.state.workhorse_idx += 1;
        self.state.workhorse[self.state.workhorse_idx] = 999999999;
        self.state.workhorse[self.state.workhorse_idx + 1] = self.state.tick;
        self.state.workhorse_idx += 2;

        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var().unwrap() as i32);

            self.state.workhorse[self.state.workhorse_idx] = 111111111;
            self.state.workhorse[self.state.workhorse_idx + 1] = entity_id;
            self.state.workhorse_idx += 2;

            if b.read_boolie().unwrap() {
                b.read_boolie();
            } else if b.read_boolie().unwrap() {
                // IF ENTITY DOES NOT EXIST
                let cls_id = b.read_nbits(9.try_into().unwrap()).unwrap();
                let serial = b.read_nbits(10);

                let mut entity = Entity {
                    class_id: cls_id,
                    entity_id: entity_id as u32,
                    //props: HashMap::default(),
                };
                self.state.entities[entity_id as usize] = (entity_id as u32, entity);
                self.update_entity(&mut b, entity_id);
                //entities[entity_id as usize] = (entity_id as u32, e);
            } else {
                // IF ENTITY DOES EXIST
                self.update_entity(&mut b, entity_id);
            }
        }
    }

    #[inline(always)]
    pub fn update_entity(&mut self, bitreader: &mut MyBitreader, entity_id: i32) {
        let mut val = -1;
        let new_way = bitreader.read_boolie().unwrap();
        let cls_id = self.state.entities[entity_id as usize].1.class_id;

        let workhorse_idx_start = self.state.workhorse_idx;
        loop {
            val = bitreader.read_inx(val, new_way).unwrap();
            if val == -1 {
                break;
            }
            self.state.workhorse[self.state.workhorse_idx] = val;
            self.state.workhorse_idx += 1;
        }
        let sv_cls = self.maps.serverclass_map.get(&(cls_id as u16)).unwrap();

        for i in workhorse_idx_start..self.state.workhorse_idx {
            let idx = self.state.workhorse[i];
            let prop = &sv_cls.props[idx as usize];
            let p = bitreader.decode(prop);
        }
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
        let pdata = b.decode(prop);
        let ok = vec![43, 41, 39, 40];
        //baseline.insert(prop.name.to_owned(), pdata);
    }
    baselines.insert(sv_cls.id.try_into().unwrap(), baseline);
}
