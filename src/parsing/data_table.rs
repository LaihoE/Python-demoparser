use crate::parsing::entities::Prop;
use crate::Demo;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_SendTable;
use protobuf;
use protobuf::Message;
use std::collections::HashSet;
use std::vec;

#[derive(Debug)]
pub struct ServerClass {
    pub id: u16,
    pub name: String,
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
            let my_id = self.read_short();
            let name = self.read_string();
            let dt = self.read_string();
            if self.parse_props {
                let props = self.flatten_dt(&self.dt_map.as_ref().unwrap()[&dt]);

                self.serverclass_map.insert(
                    my_id,
                    ServerClass {
                        id: my_id,
                        name: name,
                        dt: dt,
                        props: props,
                    },
                );
            }
        }
    }
    pub fn get_excl_props(&self, table: &CSVCMsg_SendTable) -> Vec<Sendprop_t> {
        let mut excl = vec![];

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

    pub fn flatten_dt(&self, table: &CSVCMsg_SendTable) -> Vec<Prop> {
        let excl = self.get_excl_props(table);
        let mut newp = self.get_props(table, &excl);
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
        &self,
        excl: &Vec<Sendprop_t>,
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

    pub fn get_props(&self, table: &CSVCMsg_SendTable, excl: &Vec<Sendprop_t>) -> Vec<Prop> {
        let mut flat: Vec<Prop> = Vec::new();
        let mut cnt = 0;
        for prop in &table.props {
            if (prop.flags() & (1 << 8) != 0)
                || (prop.flags() & (1 << 6) != 0)
                || self.is_prop_excl(excl, &table, prop)
            {
                continue;
            }

            if prop.type_() == 6 {
                let sub_table = &self.dt_map.as_ref().unwrap()[&prop.dt_name().to_string()];
                let child_props = self.get_props(sub_table, excl);

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
                    prop: prop.clone(),
                    arr: Some(table.props[cnt].clone()),
                    table: table.clone(),
                    col: 1,
                    data: None,
                };
                flat.push(prop_arr);
            } else {
                let prop = Prop {
                    prop: prop.clone(),
                    arr: None,
                    table: table.clone(),
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
