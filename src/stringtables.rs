use crate::protobuf::Message;
use crate::Demo;

impl Demo {
    pub fn create_string_table(&mut self, data: &[u8]) {
        let msg = Message::parse_from_bytes(data).unwrap();
        println("{:#?}", msg);
    }
}
