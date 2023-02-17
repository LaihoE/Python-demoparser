use crate::parsing::demo_parsing::*;
use crate::parsing::parser::Parser;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_SendTable;
use protobuf;
use protobuf::Message;
use smallvec::{smallvec, SmallVec};
use std::collections::HashSet;
use std::vec;

use super::read_bytes::ByteReader;
#[derive(Debug, Clone)]
pub struct ServerClass {
    pub id: u16,
    pub dt: String,
    pub props: Vec<Prop>,
}

impl Parser {
    pub fn parse_datatable(&mut self, byte_reader: &mut ByteReader) {
        /*
        Parse datatables. These are the tables that entities refer to for values. If this fails then gg?
        */
        self.state.dt_started_at = (byte_reader.byte_idx - 6) as u64;
        let _ = byte_reader.read_i32();

        loop {
            let _ = byte_reader.read_varint();
            let size = byte_reader.read_varint();
            let data = byte_reader.read_n_bytes(size);

            let table: Result<CSVCMsg_SendTable, protobuf::Error> = Message::parse_from_bytes(data);
            match table {
                Ok(table) => {
                    if table.is_end() {
                        break;
                    }

                    self.maps.dt_map.as_mut().unwrap().insert(
                        table.net_table_name.as_ref().unwrap().to_string(),
                        table.clone(),
                    );
                }
                Err(e) => {
                    panic!(
                        "Failed to parse datatable at tick {}. Error: {}",
                        self.state.tick, e
                    )
                }
            }
        }
        let class_count = byte_reader.read_short();
        for _ in 0..class_count {
            let id = byte_reader.read_short();
            let _ = byte_reader.read_string();
            let dt = byte_reader.read_string();
            // Ids for classes we use
            //if id == 275 || id == 43 || id == 41 || id == 39 || id == 40 {
            let props = self.flatten_dt(&self.maps.dt_map.as_ref().unwrap()[&dt], dt.clone());
            let server_class = ServerClass { id, dt, props };
            // Set baselines parsed earlier in stringtables.
            // Happens when stringtable, with instancebaseline, comes
            // before this event. Seems oddly complicated
            match self.maps.baseline_no_cls.get(&(id as u32)) {
                Some(user_data) => {
                    parse_baselines(&user_data, &server_class, &mut self.maps.baselines);
                    // Remove after being parsed
                    self.maps.baseline_no_cls.remove(&(id as u32));
                }
                None => {}
            }
            self.maps.serverclass_map.insert(id, server_class);
            //}
        }
    }
    pub fn get_excl_props(&self, table: &CSVCMsg_SendTable) -> SmallVec<[Sendprop_t; 32]> {
        let mut excl: SmallVec<[Sendprop_t; 32]> = smallvec![];

        for prop in &table.props {
            if prop.flags() & (1 << 6) != 0 {
                excl.push(prop.clone());
            }

            if prop.type_() == 6 {
                let sub_table = &self.maps.dt_map.as_ref().unwrap()[prop.dt_name()];
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
        /*
        excl is very short so O(n) probably best/fine
        */
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
        let mut flat: Vec<Prop> = vec![];
        let mut cnt = 0;

        for prop in &table.props {
            if (prop.flags() & (1 << 8) != 0)
                || (prop.flags() & (1 << 6) != 0)
                || self.is_prop_excl(excl, &table, prop)
            {
                continue;
            }
            if prop.type_() == 6 {
                let sub_table = &self.maps.dt_map.as_ref().unwrap()[&prop.dt_name().to_string()];
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
