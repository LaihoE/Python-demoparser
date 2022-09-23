use std::sync::Arc;

use crate::parsing::entities::Prop;
use crate::Demo;
use csgoproto::netmessages::CSVCMsg_SendTable;
use protobuf::Message;

#[derive(Debug)]
pub struct ServerClass {
    pub id: u16,
    pub name: String,
    pub dt: String,
    pub fprops: Option<Vec<Prop>>,
}
#[derive(Debug, Clone)]
pub struct server_class_blueprint {
    pub id: u16,
    pub name: String,
    pub dt: String,
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
                    self.dt_map.lock().unwrap().as_mut().unwrap().insert(
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
        let mut blueprints = vec![];
        for x in 0..class_count {
            let my_id = self.read_short();
            let name = self.read_string();
            let dt = self.read_string();
            let blueprint = server_class_blueprint {
                id: my_id,
                name: name,
                dt: dt,
            };
            blueprints.push(blueprint);
        }

        for blueprint_inx in 0..class_count {
            let blueprint = blueprints[blueprint_inx as usize].clone();
            let dt_map_clone = Arc::clone(&self.dt_map);
            let server_class_map_clone = Arc::clone(&self.serverclass_map);
            if self.parse_props {
                let map = dt_map_clone.lock().unwrap();
                let table = map.as_ref().unwrap()[&blueprint.dt].clone();
                drop(map);

                self.pool.spawn(move || {
                    Demo::flatten_dt_data_table(
                        table,
                        dt_map_clone.clone(),
                        blueprint,
                        server_class_map_clone,
                    );
                })
            }
        }
    }
}
