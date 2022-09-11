use crate::protobuf::Message;
use crate::Demo;

struct StringTable {
    name: String,
    max_entries: i32,
    udfs: i32,
    udsb: i32,
}

impl Demo {
    pub fn create_string_table(&mut self, data: &[u8]) {
        let msg = Message::parse_from_bytes(data).unwrap();
        println("{:#?}", msg);
    }
}
