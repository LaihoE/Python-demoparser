use crate::parsing::entities::Prop;
use crate::Demo;
use csgoproto::netmessages::CSVCMsg_SendTable;
use fxhash::FxHashMap;
use hashbrown::HashMap;
use protobuf::Message;

pub struct ServerClass<'a> {
    pub id: u16,
    pub name: String,
    pub dt: String,
    pub fprops: Option<Vec<Prop<'a>>>,
}

pub struct ServerClassInstructions {
    pub id: u16,
    pub name: String,
    pub dt: String,
}

impl<'a> Demo<'a> {
    pub fn parse_datatable(&mut self) -> Vec<ServerClassInstructions> {
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
        let mut server_classes = vec![];
        for _ in 0..class_count {
            let id = self.read_short();
            let name = self.read_string();
            let dt = self.read_string();
            if self.parse_props {
                let props = Demo::flatten_dt(&self.dt_map.as_ref().unwrap()[&dt], &self.dt_map);

            if self.parse_props {
                let sci = ServerClassInstructions {
                    id: id,
                    name: name,
                    dt: dt,
                };
                server_classes.push(sci);
            }
        }
        server_classes
    }
    pub fn create_serverclasses(
        instructions: Vec<ServerClassInstructions>,
        table_map: &HashMap<String, CSVCMsg_SendTable>,
    ) -> Vec<ServerClass> {
        let mut serverclasses: Vec<ServerClass> = Vec::new();
        for instruction in instructions {
            let table = &table_map[&instruction.dt];
            let props = Demo::flatten_dt(&table, table_map);

            let server_class = ServerClass {
                id: instruction.id,
                name: instruction.name,
                dt: instruction.dt,
                fprops: Some(props),
            };
            serverclasses.push(server_class);
        }
        serverclasses
    }
}
