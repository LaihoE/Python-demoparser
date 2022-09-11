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
use std::io;
use std::vec;

#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    pub serial: u32,
    //props:
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

        //println!("{}", pack_ents.entity_data().len());

        //let pp = &pack_ents.entity_data()[2..pack_ents.entity_data().len()];
        let mut b = BitReader::new(pack_ents.entity_data());
        b.ensure_bits();
        //let mut b = BitReader::new(pp);

        let mut entity_id = 0;

        for inx in 0..upd {
            if entity_id != 0 {
                entity_id += b.read_u_bit_var();
            } else {
                b.read_u_bit_var();
            }

            if b.read_bool() {
                self.entities.as_mut().unwrap().insert(entity_id, None);
                b.read_bool();
            } else if b.read_bool() {
                let cls_id = b.read_nbits(self.class_bits.try_into().unwrap());
                let serial = b.read_nbits(10);

                let new_entitiy = Entity {
                    class_id: cls_id,
                    entity_id: entity_id,
                    serial: serial,
                };

                self.entities
                    .as_mut()
                    .unwrap()
                    .insert(entity_id, Some(new_entitiy));

                self.read_new_ent(
                    &Entity {
                        class_id: cls_id,
                        entity_id: entity_id,
                        serial: serial,
                    },
                    &mut b,
                );
            } else {
                let e = &self.entities.as_ref().unwrap()[&entity_id]
                    .as_ref()
                    .unwrap();
                self.read_new_ent(&e, &mut b);
            }
        }
    }

    pub fn handle_entity_upd(&self, sv_cls: &ServerClass, b: &mut BitReader<&[u8]>) {
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

        for inx in indicies {
            /*
            println!(
                "LEN{} INX: {} var:{}",
                l,
                inx,
                &sv_cls.fprops.as_ref().unwrap()[inx as usize]
                    .prop
                    .var_name()
            );
            */
            let mut cnt = 0;
            for x in sv_cls.fprops.as_ref().unwrap() {
                //println!("CNT:{cnt} {:?}", x.prop.var_name());
                cnt += 1;
            }

            let prop = &sv_cls.fprops.as_ref().unwrap()[inx as usize];
            let r = b.decode(prop);
        }
    }

    pub fn read_new_ent(&self, ent: &Entity, b: &mut BitReader<&[u8]>) {
        let sv_cls = &self.serverclass_map[&(ent.class_id as u16)];
        self.handle_entity_upd(sv_cls, b);
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
        let (mut newp, badv) = self.get_props(table, &excl);

        let mut cnt = 0;

        let mut prios = vec![];

        //let mut newp: Vec<Prop> = Vec::new();

        for p in &newp {
            prios.push(p.prop.priority());
        }

        /*
        for mut p in fprops {
            if badv.contains(&p.prop.var_name().to_string()) {
                p.col = 1;
                newp.push(p);
            } else {
                newp.push(p);
            }
        }
        */

        //newp.sort_by_key(|x| x.col);

        if table.net_table_name() == "DT_CSPlayer" {
            for cnt in 0..newp.len() {
                let p = &newp[cnt];

                println!(
                    "pre {} {} {} {}",
                    cnt,
                    p.prop.var_name(),
                    p.col,
                    p.prop.priority()
                );
            }
            //panic!("k");
        }

        prios.dedup();
        let set: HashSet<_> = prios.drain(..).collect(); // dedup
        prios.extend(set.into_iter());
        //println!("PRIOS {:?}", prios);

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

        //fprops.sort_by_key(|x| x.prop.priority());

        if table.net_table_name() == "DT_CSPlayer" {
            for p in &newp {
                println!(
                    "REEEEEEe {} {} {} {}",
                    cnt,
                    p.prop.var_name(),
                    p.col,
                    p.prop.priority()
                );

                cnt += 1;
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

    pub fn get_props(
        &self,
        table: &CSVCMsg_SendTable,
        excl: &Vec<Sendprop_t>,
    ) -> (Vec<Prop>, Vec<String>) {
        let mut flat: Vec<Prop> = Vec::new();
        let mut cnt = 0;
        let mut badv: Vec<String> = Vec::new();
        let mut child_props = Vec::new();
        let mut sub = Vec::new();

        for prop in &table.props {
            cnt += 1;

            if (prop.flags() & (1 << 8) != 0)
                || (prop.flags() & (1 << 6) != 0)
                || self.is_prop_excl(excl.clone(), &table, prop.clone())
            {
                let found = self.is_prop_excl(excl.clone(), &table, prop.clone());
                continue;
            }

            if prop.type_() == 6 {
                let sub_table = &self.dt_map.as_ref().unwrap()[&prop.dt_name().to_string()];
                (child_props, sub) = self.get_props(sub_table, excl);
                for t in sub {
                    badv.push(t);
                }
                if (prop.flags() & (1 << 11)) == 0 {
                    for mut p in child_props {
                        badv.push(p.prop.var_name().to_string());
                        p.col = 1;
                        flat.push(p);
                    }
                } else {
                    for mut p in child_props {
                        p.col = 0;
                        flat.push(p);
                    }
                }
            } else if prop.type_() == 5 {
                let prop_arr = Prop {
                    prop: prop.clone(),
                    arr: None, //arr: Some(table.props[cnt - 1]),
                    table: table.clone(),
                    col: 0,
                };
                flat.push(prop_arr);
            } else {
                let prop = Prop {
                    prop: prop.clone(),
                    arr: None,
                    table: table.clone(),
                    col: 0,
                };
                flat.push(prop);
            }
        }
        flat.sort_by_key(|x| x.col);
        for f in &flat {
            print!("{} ", f.prop.var_name());
        }
        println!("");
        return (flat, badv);
    }
}
