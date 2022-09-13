use crate::game_events::HurtEvent;
use crate::newbitreader::Bitr;
use crate::read_bits::BitReader;
use crate::Demo;
use crate::ServerClass;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use csgoproto::netmessages::CSVCMsg_SendTable;
use protobuf;
use protobuf::Message;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io;
use std::vec;

#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    pub serial: u32,
    pub props: Vec<f32>,
}
#[derive(Debug)]
pub struct Prop {
    pub prop: Sendprop_t,
    pub arr: Option<Sendprop_t>,
    pub table: CSVCMsg_SendTable,
    pub col: i32,
}

impl Demo {
    pub fn parse_packet_entities(&mut self, pack_ents: CSVCMsg_PacketEntities) {
        let upd = pack_ents.updated_entries();

        let mut b = BitReader::new(pack_ents.entity_data());
        b.ensure_bits();

        let mut entity_id: i32 = -1;

        for inx in 0..upd {
            entity_id += 1 + (b.read_u_bit_var() as i32);
            if entity_id < 0 {
                break;
            }
            if entity_id > 100000 {
                break;
            }
            if b.read_bool() {
                /*
                self.entities
                    .as_mut()
                    .unwrap()
                    .insert(entity_id.try_into().unwrap(), None);
                */
                b.read_bool();
            } else if b.read_bool() {
                let cls_id = b.read_nbits(self.class_bits.try_into().unwrap());
                let serial = b.read_nbits(10);

                let new_entitiy = Entity {
                    class_id: cls_id,
                    entity_id: entity_id.try_into().unwrap(),
                    serial: serial,
                    props: Vec::new(),
                };

                self.entities
                    .as_mut()
                    .unwrap()
                    .insert(entity_id.try_into().unwrap(), Some(new_entitiy));

                let mut e = Entity {
                    class_id: cls_id,
                    entity_id: entity_id.try_into().unwrap(),
                    serial: serial,
                    props: Vec::new(),
                };
                let data = self.read_new_ent(&e, &mut b);
                e.props.extend(data);
            } else {
                if entity_id < 0 {
                    break;
                }
                if entity_id > 100000 {
                    break;
                }

                if !self
                    .entities
                    .as_ref()
                    .unwrap()
                    .contains_key(&(entity_id.try_into().unwrap()))
                {
                    continue;
                }

                let hm = self.entities.as_ref().unwrap();
                if hm.contains_key(&(entity_id as u32)) {
                    let ent = hm.get(&(entity_id as u32));
                    if ent.as_ref().unwrap().is_some() {
                        let x = ent.as_ref().unwrap().as_ref().unwrap();
                        //println!("{:?}", &x.props.len());

                        let data = self.read_new_ent(&x, &mut b);

                        let mut mhm = self.entities.as_mut().unwrap();
                        let mut_ent = mhm.get_mut(&(entity_id as u32));
                        let mut ps = &mut mut_ent.unwrap().as_mut().unwrap().props;
                        for d in data {
                            ps.push(d);
                        }

                        //println!("{:?}", &x.props);
                        //mut_ent.unwrap().as_mut().unwrap().props.extend(data);
                    }
                }
            }
        }
    }

    pub fn handle_entity_upd(&self, sv_cls: &ServerClass, b: &mut BitReader<&[u8]>) -> Vec<f32> {
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

        if indicies.len() > 0 {
            let y = 0;
        }
        let l = indicies.len();
        let mut data: Vec<f32> = Vec::new();
        for inx in indicies {
            let mut cnt = 0;
            if &sv_cls.fprops.as_ref().unwrap().len() > &(inx as usize) {
                let prop = &sv_cls.fprops.as_ref().unwrap()[inx as usize];
                let r = b.decode(prop);
                //println!("{}", prop.prop.var_name());
                if prop.prop.var_name() == "m_angEyeAngles[1]" {
                    data.push(r);
                    println!("{inx} {} {}", prop.prop.var_name(), r);
                }
            }
        }
        data
    }

    pub fn read_new_ent(&self, ent: &Entity, b: &mut BitReader<&[u8]>) -> Vec<f32> {
        let mut data = vec![];
        if self.serverclass_map.contains_key(&(ent.class_id as u16)) {
            let sv_cls = &self.serverclass_map[&(ent.class_id as u16)];
            let props = self.handle_entity_upd(sv_cls, b);
            data.extend(props);
        }
        data
    }

    pub fn get_excl_props(&self, table: &CSVCMsg_SendTable) -> Vec<Sendprop_t> {
        let mut excl = vec![];

        for prop in &table.props {
            if (prop.flags() & (1 << 6) != 0) {
                excl.push(prop.clone());
            }

            if prop.type_() == 6 {
                let sub_table = &self.dt_map.as_ref().unwrap()[prop.dt_name()];
                excl.extend(self.get_excl_props(&sub_table.clone()));
            }
        }
        excl
    }

    pub fn flatten_dt(&self, table: &CSVCMsg_SendTable) -> Vec<Prop> {
        let excl = self.get_excl_props(table);
        let mut newp = self.get_props(table, &excl);

        let mut cnt = 0;

        let mut prios = vec![];

        //let mut newp: Vec<Prop> = Vec::new();

        for p in &newp {
            prios.push(p.prop.priority());
        }

        prios.dedup();
        let set: HashSet<_> = prios.drain(..).collect(); // dedup
        prios.extend(set.into_iter());

        prios.push(64);
        let mut start = 0;
        prios.sort();

        for prio_inx in 0..prios.len() {
            let mut priority = prios[prio_inx];
            loop {
                let mut currentprop = start;
                while currentprop < newp.len() {
                    let prop = newp[currentprop].prop.clone();
                    if (prop.priority() == priority
                        || (priority == 64 && ((prop.flags() & (1 << 18)) != 0)))
                    {
                        if (start != currentprop) {
                            newp.swap(start, currentprop);
                        }
                        start += 1;
                    }
                    currentprop += 1;
                }
                if (currentprop == newp.len()) {
                    break;
                }
            }
        }

        newp
    }

    #[inline]
    pub fn is_prop_excl(
        &self,
        excl: Vec<Sendprop_t>,
        table: &CSVCMsg_SendTable,
        prop: Sendprop_t,
    ) -> bool {
        for item in excl {
            if table.net_table_name() == item.dt_name() && prop.var_name() == item.var_name() {
                return true;
            }
        }
        false
    }

    pub fn get_props(&self, table: &CSVCMsg_SendTable, excl: &Vec<Sendprop_t>) -> Vec<Prop> {
        let mut flat: Vec<Prop> = Vec::new();
        let mut child_props = Vec::new();
        let mut cnt = 0;
        for prop in &table.props {
            if (prop.flags() & (1 << 8) != 0)
                || (prop.flags() & (1 << 6) != 0)
                || self.is_prop_excl(excl.clone(), &table, prop.clone())
            {
                let found = self.is_prop_excl(excl.clone(), &table, prop.clone());
                continue;
            }

            if prop.type_() == 6 {
                let sub_table = &self.dt_map.as_ref().unwrap()[&prop.dt_name().to_string()];
                child_props = self.get_props(sub_table, excl);

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
                    prop: prop.clone(),
                    arr: Some(table.props[cnt - 1].clone()),
                    table: table.clone(),
                    col: 1,
                };
                flat.push(prop_arr);
            } else {
                let prop = Prop {
                    prop: prop.clone(),
                    arr: None,
                    table: table.clone(),
                    col: 1,
                };
                flat.push(prop);
            }
            cnt += 1;
        }
        flat.sort_by_key(|x| x.col);
        return flat;
    }
}
