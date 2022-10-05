use crate::parsing::data_table::ServerClass;
use crate::parsing::game_events::HurtEvent;
use crate::parsing::read_bits::BitReader;
use crate::parsing::read_bits::PropAtom;
use crate::parsing::read_bits::PropData;
use crate::Demo;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use csgoproto::netmessages::CSVCMsg_SendTable;
use fxhash::FxHashMap;
use hashbrown::HashMap;
use protobuf;
use protobuf::Message;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io;
use std::rc::Rc;
use std::vec;

#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    pub serial: u32,
    pub props: HashMap<String, PropAtom>,
}

#[derive(Debug)]
pub struct Prop {
    pub prop: Sendprop_t,
    pub arr: Option<Sendprop_t>,
    pub table: CSVCMsg_SendTable,
    pub col: i32,
    pub data: Option<PropData>,
}

#[inline]
pub fn is_wanted_prop(this_prop: &Prop, wanted_props: &Vec<String>) -> bool {
    let this_prop_name = this_prop.prop.var_name();
    for prop in wanted_props {
        if prop == this_prop_name {
            return true;
        }
    }
    false
}

impl Demo {
    pub fn parse_packet_entities(&mut self, pack_ents: CSVCMsg_PacketEntities, should_parse: bool) {
        let n_upd_ents = pack_ents.updated_entries();
        let left_over = (pack_ents.entity_data().len() % 4) as i32;
        let mut b = BitReader::new(pack_ents.entity_data(), left_over);
        b.read_uneven_end_bits();
        let mut entity_id: i32 = -1;

        for _ in 0..n_upd_ents {
            entity_id += 1 + (b.read_u_bit_var() as i32);

            if b.read_bool() {
                b.read_bool();
            } else if b.read_bool() {
                let cls_id = b.read_nbits(self.class_bits.try_into().unwrap());
                let serial = b.read_nbits(10);

                let new_entitiy = Entity {
                    class_id: cls_id,
                    entity_id: entity_id.try_into().unwrap(),
                    serial: serial,
                    props: HashMap::default(),
                };

                self.entities
                    .as_mut()
                    .unwrap()
                    .insert(entity_id.try_into().unwrap(), Some(new_entitiy));

                let mut e = Entity {
                    class_id: cls_id,
                    entity_id: entity_id.try_into().unwrap(),
                    serial: serial,
                    props: HashMap::default(),
                };

                let data = self.read_new_ent(&e, &mut b);

                for pa in data {
                    if self.wanted_props.contains(&pa.prop_name) {
                        e.props.insert(pa.prop_name.clone(), pa);
                    }
                }
            } else {
                let hm = self.entities.as_ref().unwrap();
                let ent = hm.get(&(entity_id.try_into().unwrap()));

                if ent.as_ref().unwrap().is_some() {
                    let ent = ent.as_ref().unwrap().as_ref().unwrap();
                    let data = self.read_new_ent(&ent, &mut b);

                    let mhm = self.entities.as_mut().unwrap();
                    let mut_ent = mhm.get_mut(&(entity_id as u32));

                    let e = &mut mut_ent.unwrap().as_mut().unwrap().props;

                    for pa in data {
                        if self.wanted_props.contains(&pa.prop_name) {
                            e.insert(pa.prop_name.clone(), pa);
                        }
                    }
                } else {
                    println!("ENTITY: {} NOT FOUND!", entity_id);
                    panic!("f");
                }
            }
        }
    }

    pub fn handle_entity_upd(
        &self,
        sv_cls: &ServerClass,
        b: &mut BitReader<&[u8]>,
    ) -> Vec<PropAtom> {
        let mut val = -1;
        let new_way = b.read_bool();
        let mut indicies = Vec::with_capacity(128);

        loop {
            val = b.read_inx(val, new_way);
            if val == -1 {
                break;
            }
            indicies.push(val);
        }

        let mut props: Vec<PropAtom> = Vec::with_capacity(self.wanted_props.len() + 3);

        for inx in indicies {
            let prop = &sv_cls.fprops[inx as usize];
            let pdata = b.decode(prop);

            if !is_wanted_prop(prop, &self.wanted_props) {
                continue;
            }

            match pdata {
                PropData::VecXY(v) => {
                    let endings = ["_X", "_Y"];
                    for inx in 0..2 {
                        let data = PropData::F32(v[inx]);
                        let name = prop.prop.var_name().to_string() + endings[inx];
                        let atom = PropAtom {
                            prop_name: name,
                            data: data,
                            tick: self.tick,
                        };
                        props.push(atom);
                    }
                }
                PropData::VecXYZ(v) => {
                    let endings = ["_X", "_Y", "_Z"];
                    for inx in 0..3 {
                        let data = PropData::F32(v[inx]);
                        let name = prop.prop.var_name().to_string() + endings[inx];
                        let atom = PropAtom {
                            prop_name: name,
                            data: data,
                            tick: self.tick,
                        };
                        props.push(atom);
                    }
                }

                _ => {
                    let atom = PropAtom {
                        prop_name: prop.prop.var_name().to_string(),
                        data: pdata,
                        tick: self.tick,
                    };
                    props.push(atom);
                }
            }
        }
        //println!("{:?}", props);
        props
    }

    pub fn read_new_ent(&self, ent: &Entity, b: &mut BitReader<&[u8]>) -> Vec<PropAtom> {
        let sv_cls = &self.serverclass_map[&(ent.class_id.try_into().unwrap())];
        let props = self.handle_entity_upd(sv_cls, b);
        props
    }
}
