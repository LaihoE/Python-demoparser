use std::sync::Arc;

use crate::parsing::entities::Prop;
use crate::Demo;
use csgoproto::netmessages::CSVCMsg_SendTable;
use protobuf::Message;

pub struct ServerClass {
    pub id: u16,
    pub name: String,
    pub dt: String,
    pub fprops: Option<Vec<Prop>>,
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

        for _ in 0..class_count {
            let my_id = self.read_short();
            let name = self.read_string();
            let dt = self.read_string();
            let dt_map_clone = Arc::clone(&self.dt_map);

            if self.parse_props {
                let props = Demo::flatten_dt(
                    dt_map_clone.lock().unwrap().as_ref().unwrap()[&dt].clone(),
                    dt_map_clone.clone(),
                );

                let server_class = ServerClass {
                    id: my_id,
                    name: name,
                    dt: dt,
                    fprops: None,
                };

                self.serverclass_map
                    .lock()
                    .unwrap()
                    .insert(server_class.id, server_class);
            }
        }
    }
}
