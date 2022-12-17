use super::parser::JobResult;
use super::parser::ParsingMaps;
use super::read_bytes::ByteReader;
use crate::parsing::entities::parse_baselines;
use crate::parsing::entities::Prop;
use crate::parsing::parser::Parser;
use ahash::HashMap;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_SendTable;
use protobuf;
use protobuf::Message;
use smallvec::{smallvec, SmallVec};
use std::collections::HashSet;
use std::default;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;
use std::vec;

#[derive(Debug, Clone)]
pub struct ServerClass {
    pub id: u16,
    pub dt: String,
    pub props: Vec<Prop>,
}
#[derive(Debug, Clone)]
pub struct ServerClasses {
    pub player: ServerClass,
    pub world: ServerClass,
}

impl Parser {
    pub fn parse_datatable(
        byte_reader: &mut ByteReader,
        parsing_maps: Arc<RwLock<ParsingMaps>>,
    ) -> JobResult {
        /*
        Parse datatables. These are the tables that entities refer to for values. If this fails then gg?
        */
        let before = Instant::now();

        let mut dt_map: HashMap<String, CSVCMsg_SendTable> = HashMap::default();
        let skipbytes = byte_reader.read_i32();
        //byte_reader.skip_n_bytes(skipbytes as u32);
        //return JobResult::None;
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
                    dt_map.insert(
                        table.net_table_name.as_ref().unwrap().to_string(),
                        table.clone(),
                    );
                }
                Err(e) => {
                    panic!("Failed to parse datatable. Error: {}", e)
                }
            }
        }
        let class_count = byte_reader.read_short();
        let mut player: Option<ServerClass> = None;
        let mut world: Option<ServerClass> = None;

        for _ in 0..class_count {
            let id = byte_reader.read_short();
            let _ = byte_reader.read_string();
            let dt = byte_reader.read_string();

            match id {
                275 => {
                    let props = Parser::flatten_dt(&dt_map[&dt], dt.clone(), &dt_map);
                    let server_class = ServerClass { id, dt, props };
                    world = Some(server_class);
                }
                40 => {
                    let props = Parser::flatten_dt(&dt_map[&dt], dt.clone(), &dt_map);
                    let server_class = ServerClass { id, dt, props };
                    player = Some(server_class)
                }
                _ => {}
            }
        }
        if player.is_some() && world.is_some() {
            let svcs = ServerClasses {
                player: player.unwrap(),
                world: world.unwrap(),
            };
            let mut parsing_map_write = parsing_maps.write().unwrap();
            parsing_map_write.serverclass_map = Some(svcs);
            drop(parsing_map_write);
            JobResult::None
        } else {
            panic!("FAILED TO CREATE SERVERCLASS MAP")
        }
    }
    pub fn get_excl_props(
        table: &CSVCMsg_SendTable,
        dt_map: &HashMap<String, CSVCMsg_SendTable>,
    ) -> SmallVec<[Sendprop_t; 32]> {
        let mut excl: SmallVec<[Sendprop_t; 32]> = smallvec![];

        for prop in &table.props {
            if prop.flags() & (1 << 6) != 0 {
                excl.push(prop.clone());
            }

            if prop.type_() == 6 {
                let sub_table = &dt_map[prop.dt_name()];
                excl.extend(Parser::get_excl_props(&sub_table.clone(), dt_map));
            }
        }
        excl
    }

    pub fn flatten_dt(
        table: &CSVCMsg_SendTable,
        table_id: String,
        dt_map: &HashMap<String, CSVCMsg_SendTable>,
    ) -> Vec<Prop> {
        let excl = Parser::get_excl_props(table, dt_map);
        let mut newp = Parser::get_props(table, table_id, &excl, dt_map);
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
        excl: &SmallVec<[Sendprop_t; 32]>,
        table: &CSVCMsg_SendTable,
        prop: &Sendprop_t,
        dt_map: &HashMap<String, CSVCMsg_SendTable>,
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
        table: &CSVCMsg_SendTable,
        table_name: String,
        excl: &SmallVec<[Sendprop_t; 32]>,
        dt_map: &HashMap<String, CSVCMsg_SendTable>,
    ) -> Vec<Prop> {
        let mut flat: Vec<Prop> = Vec::new();
        let mut cnt = 0;

        for prop in &table.props {
            if (prop.flags() & (1 << 8) != 0)
                || (prop.flags() & (1 << 6) != 0)
                || Parser::is_prop_excl(excl, &table, prop, dt_map)
            {
                continue;
            }
            if prop.type_() == 6 {
                let sub_table = &dt_map[&prop.dt_name().to_string()];
                let child_props =
                    Parser::get_props(sub_table, prop.dt_name().to_string(), excl, dt_map);

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
