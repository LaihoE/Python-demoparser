use crate::parsing::parser::Parser;
use pyo3::Py;
use std::collections::HashMap;
use std::convert::TryInto;
use std::str;
//use hashbrown::HashMap;
use pyo3::PyAny;
use pyo3::ToPyObject;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Header {
    pub header_magic: String,
    pub protocol: i32,
    pub network_protocol: u32,
    pub server_name: String,
    pub client_name: String,
    pub map_name: String,
    pub game_dir: String,
    pub playback_time: f32,
    pub playback_ticks: i32,
    pub playback_frames: i32,
    pub signon_length: i32,
}

impl Header {
    fn to_hashmap(&self) -> HashMap<String, String> {
        let mut hm: HashMap<String, String> = HashMap::new();
        hm.insert("protocol".to_string(), self.protocol.to_string());
        hm.insert(
            "network_protocol".to_string(),
            self.network_protocol.to_string(),
        );
        hm.insert("server_name".to_string(), self.server_name.to_string());
        hm.insert("client_name".to_string(), self.client_name.to_string());
        hm.insert("map_name".to_string(), self.map_name.to_string());
        hm.insert("game_dir".to_string(), self.game_dir.to_string());
        hm.insert("playback_time".to_string(), self.playback_time.to_string());
        hm.insert(
            "protoplayback_tickscol".to_string(),
            self.playback_ticks.to_string(),
        );
        hm.insert(
            "playback_frames".to_string(),
            self.playback_frames.to_string(),
        );
        hm.insert("signon_length".to_string(), self.signon_length.to_string());
        hm
    }
    pub fn to_py_hashmap(&self) -> Py<PyAny> {
        let hm = self.to_hashmap();
        pyo3::Python::with_gil(|py| hm.to_object(py))
    }
}

impl Parser {
    pub fn parse_demo_header(&mut self) -> Header {
        let h = Header {
            header_magic: str::from_utf8(&self.bytes[..8])
                .unwrap()
                .trim_end_matches("\x00")
                .to_string(),
            protocol: i32::from_le_bytes(self.bytes[8..12].try_into().unwrap()),
            network_protocol: u32::from_le_bytes(self.bytes[12..16].try_into().unwrap()),
            server_name: str::from_utf8(&self.bytes[16..276])
                .unwrap()
                .to_string()
                .trim_end_matches("\x00")
                .to_string(),
            client_name: str::from_utf8(&self.bytes[276..536])
                .unwrap()
                .to_string()
                .trim_end_matches("\x00")
                .to_string(),
            map_name: str::from_utf8(&self.bytes[536..796])
                .unwrap()
                .to_string()
                .trim_end_matches("\x00")
                .to_string(),
            game_dir: str::from_utf8(&self.bytes[796..1056])
                .unwrap()
                .to_string()
                .trim_end_matches("\x00")
                .to_string(),
            playback_time: f32::from_le_bytes(self.bytes[1056..1060].try_into().unwrap()),
            playback_ticks: i32::from_le_bytes(self.bytes[1060..1064].try_into().unwrap()),
            playback_frames: i32::from_le_bytes(self.bytes[1064..1068].try_into().unwrap()),
            signon_length: i32::from_le_bytes(self.bytes[1068..1072].try_into().unwrap()),
        };
        self.state.fp += 1072_usize;
        h
    }
}
