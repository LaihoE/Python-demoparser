use crate::parsing::entities::Prop;
use crate::Demo;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_SendTable;
use protobuf;
use protobuf::Message;
use smallvec::{smallvec, SmallVec};
use std::collections::HashSet;
use std::vec;

#[derive(Debug)]
pub struct ServerClass {
    pub id: u16,
    pub dt: String,
    pub props: Vec<Prop>,
}

impl Demo {
    pub fn parse_datatable(&mut self) {
        let _ = self.read_i32();
        loop {
            let _ = self.read_varint();
            let size = self.read_varint();
            let data = self.read_n_bytes(size);

            let table = Message::parse_from_bytes(data);
            match table {
                Ok(t) => {
                    let table: CSVCMsg_SendTable = t;
                    if table.is_end() {
                        break;
                    }

                    self.dt_map.as_mut().unwrap().insert(
                        table.net_table_name.as_ref().unwrap().to_string(),
                        table.clone(),
                    );
                }
                Err(e) => {
                    panic!(
                        "Failed to parse datatable at tick {}. Error: {}",
                        self.tick, e
                    )
                }
            }
        }

        let class_count = self.read_short();
        self.class_bits = (class_count as f32 + 1.).log2().ceil() as u32;

        for _ in 0..class_count {
            let id = self.read_short();
            let _ = self.read_string();
            let dt = self.read_string();

            if self.parse_props {
                let props = self.flatten_dt(&self.dt_map.as_ref().unwrap()[&dt], dt.clone());
                self.serverclass_map
                    .insert(id, ServerClass { id, dt, props });
            }
        }
    }
    pub fn get_excl_props(&self, table: &CSVCMsg_SendTable) -> SmallVec<[Sendprop_t; 32]> {
        let mut excl: SmallVec<[Sendprop_t; 32]> = smallvec![];

        for prop in &table.props {
            if prop.flags() & (1 << 6) != 0 {
                excl.push(prop.clone());
            }

            if prop.type_() == 6 {
                let sub_table = &self.dt_map.as_ref().unwrap()[prop.dt_name()];
                excl.extend(self.get_excl_props(&sub_table.clone()));
            }
        }
        excl
    }

    pub fn flatten_dt(&self, table: &CSVCMsg_SendTable, table_id: String) -> Vec<Prop> {
        let excl = self.get_excl_props(table);
        let mut newp = self.get_props(table, table_id, &excl);
        let mut prios = vec![];
        for p in &newp {
            prios.push(p.priority);
        }

        let set: HashSet<_> = prios.drain(..).collect();
        prios.extend(set.into_iter());
        prios.push(64);
        prios.sort();
        let mut start = 0;

        for prio in prios {
            loop {
                let mut currentprop = start;
                while currentprop < newp.len() {
                    let prop = &newp[currentprop];
                    if prop.priority == prio || prio == 64 && ((prop.flags & (1 << 18)) != 0) {
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
        &self,
        excl: &SmallVec<[Sendprop_t; 32]>,
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

    pub fn get_props(
        &self,
        table: &CSVCMsg_SendTable,
        table_name: String,
        excl: &SmallVec<[Sendprop_t; 32]>,
    ) -> Vec<Prop> {
        let mut flat: Vec<Prop> = Vec::new();
        let mut cnt = 0;
        //println!("{}", table_id);

        for prop in &table.props {
            if (prop.flags() & (1 << 8) != 0)
                || (prop.flags() & (1 << 6) != 0)
                || self.is_prop_excl(excl, &table, prop)
            {
                continue;
            }
            //println!("DTN {}", prop.dt_name());
            if prop.type_() == 6 {
                //println!("{}", prop.dt_name());
                let sub_table = &self.dt_map.as_ref().unwrap()[&prop.dt_name().to_string()];
                let child_props = self.get_props(sub_table, prop.dt_name().to_string(), excl);

                if (prop.flags() & (1 << 11)) == 0 {
                    for mut p in child_props {
                        p.col = 0;
                        flat.push(p);
                    }
                } else {
                    for p in child_props {
                        flat.push(p);
                    }
                }
            } else if prop.type_() == 5 {
                let prop_arr = Prop {
                    table: table_name.clone(),
                    name: prop.var_name().to_string(),
                    arr: Some(table.props[cnt].clone()),
                    col: 1,
                    data: None,
                    flags: prop.flags(),
                    num_elements: prop.num_elements(),
                    num_bits: prop.num_bits(),
                    low_value: prop.high_value(),
                    high_value: prop.high_value(),
                    priority: prop.priority(),
                    p_type: prop.type_(),
                };
                //println!("{} {}", prop.dt_name(), prop.var_name());
                flat.push(prop_arr);
            } else {
                let prop = Prop {
                    name: prop.var_name().to_string(),
                    table: table_name.clone(),
                    arr: None,
                    col: 1,
                    data: None,
                    flags: prop.flags(),
                    num_elements: prop.num_elements(),
                    num_bits: prop.num_bits(),
                    low_value: prop.high_value(),
                    high_value: prop.high_value(),
                    priority: prop.priority(),
                    p_type: prop.type_(),
                };
                flat.push(prop);
            }
            cnt += 1;
        }
        flat.sort_by_key(|x| x.col);
        return flat;
    }
}
