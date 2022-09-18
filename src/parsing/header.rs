use crate::Demo;
use std::str;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Header<'a> {
    header_magic: &'a str,
    protocol: i32,
    network_protocol: u32,
    server_name: &'a str,
    client_name: &'a str,
    map_name: &'a str,
    game_dir: &'a str,
    playback_time: f32,
    playback_ticks: i32,
    playback_frames: i32,
    signon_length: i32,
}

impl Demo<'_> {
    pub fn parse_header(&mut self) -> Header {
        let h = Header {
            header_magic: str::from_utf8(&self.bytes[..8]).unwrap(),
            protocol: i32::from_le_bytes(self.bytes[8..12].try_into().unwrap()),
            network_protocol: u32::from_le_bytes(self.bytes[12..16].try_into().unwrap()),
            server_name: str::from_utf8(&self.bytes[16..276]).unwrap(),
            client_name: str::from_utf8(&self.bytes[276..536]).unwrap(),
            map_name: str::from_utf8(&self.bytes[536..796]).unwrap(),
            game_dir: str::from_utf8(&self.bytes[796..1056]).unwrap(),
            playback_time: f32::from_le_bytes(self.bytes[1056..1060].try_into().unwrap()),
            playback_ticks: i32::from_le_bytes(self.bytes[1060..1064].try_into().unwrap()),
            playback_frames: i32::from_le_bytes(self.bytes[1064..1068].try_into().unwrap()),
            signon_length: i32::from_le_bytes(self.bytes[1068..1072].try_into().unwrap()),
        };
        self.fp += 1072 as usize;
        h
    }
}
