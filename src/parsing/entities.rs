use crate::parsing::data_table::ServerClass;
use crate::parsing::game_events::HurtEvent;
use crate::parsing::newbitreader::Bitr;
use crate::parsing::read_bits::BitReader;
use crate::parsing::read_bits::PropAtom;
use crate::parsing::read_bits::PropData;
use crate::Demo;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use csgoproto::netmessages::CSVCMsg_SendTable;
use fxhash::FxHashMap;
use protobuf;
use protobuf::Message;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io;
use std::vec;

use hashbrown::HashMap;

#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    pub serial: u32,
    //pub props: HashMap<String, PropAtom>,
    pub props: HashMap<String, PropAtom>,
}

#[derive(Debug)]
pub struct Prop<'a> {
    pub prop: &'a Sendprop_t,
    pub arr: Option<&'a Sendprop_t>,
    pub table: &'a CSVCMsg_SendTable,
    pub col: i32,
    pub data: Option<&'a PropData>,
}

impl<'a> Demo<'a> {
    pub fn parse_packet_entities(&mut self, pack_ents: CSVCMsg_PacketEntities, should_parse: bool) {
        /*
        if !should_parse {
            return;
        };
        */
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
                    let x = ent.as_ref().unwrap().as_ref().unwrap();
                    let data = self.read_new_ent(&x, &mut b);

                    let mut mhm = self.entities.as_mut().unwrap();
                    let mut_ent = mhm.get_mut(&(entity_id as u32));

                    let mut ps = &mut mut_ent.unwrap().as_mut().unwrap().props;
                    self.cnt += ps.len() as i32;

                    for pa in data {
                        if self.wanted_props.contains(&pa.prop_name) {
                            ps.insert(pa.prop_name.clone(), pa);
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
        let mut indicies = vec![];

        loop {
            val = b.read_inx(val, new_way);
            if val == -1 {
                break;
            }
            indicies.push(val);
        }

        let mut props: Vec<PropAtom> = Vec::with_capacity(indicies.len());

        for inx in indicies {
            let prop = &sv_cls.fprops.as_ref().unwrap()[inx as usize];
            let pdata = b.decode(prop);
            if !self
                .wanted_props
                .contains(&prop.prop.var_name().to_string())
            {
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

                PropData::String(_) => {}
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
        props
    }

    pub fn read_new_ent(&self, ent: &Entity, b: &mut BitReader<&[u8]>) -> Vec<PropAtom> {
        let mut data = vec![];
        let sv_cls = &self.serverclass_map[&(ent.class_id.try_into().unwrap())];
        let props = self.handle_entity_upd(sv_cls, b);
        data.extend(props);
        data
    }

    pub fn get_excl_props(
        table: &CSVCMsg_SendTable,
        table_map: &HashMap<String, CSVCMsg_SendTable>,
    ) -> Vec<Sendprop_t> {
        let mut excl = vec![];

        for prop in &table.props {
            if prop.flags() & (1 << 6) != 0 {
                excl.push(prop.clone());
            }

            if prop.type_() == 6 {
                let sub_table = &table_map[prop.dt_name()];
                excl.extend(Demo::get_excl_props(&sub_table.clone(), table_map));
            }
        }
        excl
    }

    pub fn flatten_dt<'b>(
        table: &'a CSVCMsg_SendTable,
        table_map: &'a HashMap<String, CSVCMsg_SendTable>,
    ) -> Vec<Prop<'a>> {
        let excl = Demo::get_excl_props(table, table_map);
        let mut newp = Demo::get_props(table, excl, table_map);
        let mut prios = vec![];
        for p in &newp {
            prios.push(p.prop.priority());
        }

        let set: HashSet<_> = prios.drain(..).collect();
        prios.extend(set.into_iter());
        prios.push(64);
        prios.sort();
        let mut start = 0;

        for prio_inx in 0..prios.len() {
            let priority = prios[prio_inx];
            loop {
                let mut currentprop = start;
                while currentprop < newp.len() {
                    let prop = newp[currentprop].prop.clone();
                    if prop.priority() == priority
                        || priority == 64 && ((prop.flags() & (1 << 18)) != 0)
                    {
                        if start != currentprop {
                            newp.swap(start, currentprop);
                        }
                        start += 1;
                    }
                    currentprop += 1;
                }
                if currentprop == newp.len() {
                    break;
                }
            }
        }
        newp
    }

    #[inline]
    pub fn is_prop_excl(
        excl: Vec<Sendprop_t>,
        table: &CSVCMsg_SendTable,
        prop: &Sendprop_t,
    ) -> bool {
        for item in excl {
            if table.net_table_name() == item.dt_name() && prop.var_name() == item.var_name() {
                return true;
            }
        }
        false
    }

    pub fn get_props<'b>(
        table: &'a CSVCMsg_SendTable,
        excl: Vec<Sendprop_t>,
        table_map: &'a HashMap<String, CSVCMsg_SendTable>,
    ) -> Vec<Prop<'a>> {
        let mut flat: Vec<Prop> = Vec::new();
        let mut child_props = Vec::new();
        let mut cnt = 0;
        for prop in &table.props {
            if (prop.flags() & (1 << 8) != 0)
                || (prop.flags() & (1 << 6) != 0)
                || Demo::is_prop_excl(excl.clone(), &table, prop)
            {
                continue;
            }

            if prop.type_() == 6 {
                //let sub_table = &self.dt_map.as_ref().unwrap()[&prop.dt_name().to_string()];
                let sub_table = &table_map[&prop.dt_name().to_string()];
                child_props = Demo::get_props(sub_table, excl.clone(), table_map);

                if (prop.flags() & (1 << 11)) == 0 {
                    for mut p in child_props {
                        p.col = 0;
                        flat.push(p);
                    }
                } else {
                    for mut p in child_props {
                        flat.push(p);
                    }
                }
            } else if prop.type_() == 5 {
                let prop_arr = Prop {
                    prop: prop,
                    arr: Some(&table.props[cnt]),
                    table: table,
                    col: 1,
                    data: None,
                };
                flat.push(prop_arr);
            } else {
                let prop = Prop {
                    prop: prop,
                    arr: None,
                    table: table,
                    col: 1,
                    data: None,
                };
                flat.push(prop);
            }
            cnt += 1;
        }
        flat.sort_by_key(|x| x.col);
        return flat;
    }
}
