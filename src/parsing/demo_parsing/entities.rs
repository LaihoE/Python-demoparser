use super::ByteReader;
use crate::parsing::demo_parsing::data_table::ServerClass;
use crate::parsing::demo_parsing::read_bits::MyBitreader;
use crate::parsing::variants::PropAtom;
use crate::parsing::variants::PropData;
use crate::Parser;
use ahash::HashMap;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use protobuf::Message;
use std::collections::HashSet;
use std::convert::TryInto;

//use super::stringtables::UserInfo;

static forbidden: &'static [i32] = &[0, 1, 2, 37, 103, 93, 59, 58, 40, 41, 26, 27];

#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: i32,
    pub props: Vec<Option<PropData>>,
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
pub struct EidClsHistoryEntry {
    pub eid: i32,
    pub cls_id: u32,
    pub tick: i32,
}

impl Parser {
    pub fn parse_packet_entities(&mut self, byte_reader: &mut ByteReader, size: usize) {
        let wanted_bytes = &self.bytes[byte_reader.byte_idx..byte_reader.byte_idx + size as usize];
        byte_reader.skip_n_bytes(size.try_into().unwrap());
        let pack_ents: CSVCMsg_PacketEntities = Message::parse_from_bytes(wanted_bytes).unwrap();

        let n_upd_ents = pack_ents.updated_entries();
        let mut b = MyBitreader::new(pack_ents.entity_data());
        let mut entity_id: i32 = -1;

        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var().unwrap() as i32);
            if entity_id > 64 {
                break;
            }

            if b.read_boolie().unwrap() {
                b.read_boolie();
            } else if b.read_boolie().unwrap() {
                // IF ENTITY DOES NOT EXIST
                let cls_id = b.read_nbits(9.try_into().unwrap()).unwrap();
                let serial = b.read_nbits(10);

                let mut entity = Entity {
                    class_id: cls_id,
                    entity_id: entity_id,
                    props: vec![None; 10000],
                };
                self.state.eid_cls_history.push(EidClsHistoryEntry {
                    eid: entity_id,
                    cls_id: cls_id,
                    tick: self.state.tick,
                });
                self.state.entities.insert(entity_id, entity);
                self.update_entity(&mut b, entity_id);
            } else {
                // IF ENTITY DOES EXIST
                self.update_entity(&mut b, entity_id);
            }
        }
    }
    fn find_cls_id(history: &Vec<EidClsHistoryEntry>, entity_id: i32, tick: i32) -> u32 {
        // Finds current cls id for entity. Has to be mapped based on tick because
        // entity ids are reused :(
        let myid: Vec<&EidClsHistoryEntry> =
            history.iter().filter(|x| x.eid == entity_id).collect();
        if myid.len() == 0 {
            panic!("ENITD {} NO CLS", entity_id);
        }
        if myid.len() == 1 {
            return myid[0].cls_id;
        } else {
            for e in myid.windows(2) {
                if e[1].tick > tick && e[0].tick <= tick {
                    return e[0].cls_id;
                }
            }
            return myid.last().unwrap().cls_id;
        }
    }
    fn create_ent_if_not_exist(&mut self, entity_id: i32) {
        match self.state.entities.get(&entity_id) {
            Some(_e) => {}
            None => {
                let cls_id =
                    Parser::find_cls_id(&self.state.eid_cls_history, entity_id, self.state.tick);
                let entity = Entity {
                    class_id: cls_id,
                    entity_id: entity_id,
                    props: vec![None; 10000],
                };
                self.state.entities.insert(entity_id, entity);
            }
        }
    }

    #[inline(always)]
    pub fn update_entity(&mut self, bitreader: &mut MyBitreader, entity_id: i32) {
        let mut val = -1;
        let new_way = bitreader.read_boolie().unwrap();
        let mut idx = 0;

        self.create_ent_if_not_exist(entity_id);
        let mut entity = self.state.entities.get_mut(&entity_id).unwrap();

        loop {
            val = bitreader.read_inx(val, new_way).unwrap();
            if val == -1 {
                break;
            }
            if !forbidden.contains(&val) && entity_id < 75 {
                self.state
                    .test
                    .entry(entity.class_id)
                    .or_insert(HashMap::default())
                    .entry(val as u32)
                    .or_insert(vec![])
                    .push([
                        self.state.tick,
                        self.state.frame_started_at as i32,
                        entity_id,
                    ]);
            }
            self.state.workhorse[idx] = val;
            idx += 1;
        }

        let cls_id = Parser::find_cls_id(&self.state.eid_cls_history, entity_id, self.state.tick);
        let sv_cls = self.maps.serverclass_map.get(&(cls_id as u16)).unwrap();

        for i in 0..idx {
            let idx = self.state.workhorse[i];
            let prop = &sv_cls.props[idx as usize];
            let p = bitreader.decode(prop);
            if prop.name.contains("m_iHealth") {
                println!("{} {} {} {:?}", idx, prop.name, self.state.tick, p);
            }

            entity.props[idx as usize] = p;
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
