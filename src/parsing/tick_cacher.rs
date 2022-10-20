use csgoproto::netmessages::CSVCMsg_PacketEntities;
use protobuf::Message;

use crate::parsing::variants::PropAtom;

use std::collections::HashMap;
pub struct TickCache {
    pub cache: HashMap<i32, (usize, usize)>,
}
/*
Currently not in use. Used for seeking trough packetents.
*/

impl TickCache {
    pub fn new() -> Self {
        TickCache {
            cache: HashMap::default(),
        }
    }

    pub fn insert_slice(&mut self, tick: i32, left_inx: usize, right_inx: usize) {
        self.cache.insert(tick, (left_inx, right_inx));
    }

    pub fn backtrack_prop(&self, bytes: &[u8], starting_tick: i32) {
        loop {
            let msg: CSVCMsg_PacketEntities = Message::parse_from_bytes(bytes).unwrap();
        }
    }
}
