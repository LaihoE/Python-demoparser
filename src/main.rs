use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::time::Instant;
use std::str;
use std::fs;

#[derive(Debug)]
struct Header <'a>{
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

fn parse_header(bytes: &[u8]) -> Header{   
    let h = Header{
        header_magic: str::from_utf8(&bytes[..8]).unwrap(),
        protocol: i32::from_le_bytes(bytes[8..12].try_into().unwrap()),
        network_protocol: u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
        server_name: str::from_utf8(&bytes[16..276]).unwrap(),
        client_name: str::from_utf8(&bytes[276..536]).unwrap(),
        map_name: str::from_utf8(&bytes[536..796]).unwrap(),
        game_dir: str::from_utf8(&bytes[796..1056]).unwrap(),
        playback_time: f32::from_le_bytes(bytes[1056..1060].try_into().unwrap()),
        playback_ticks: i32::from_le_bytes(bytes[1060..1064].try_into().unwrap()),
        playback_frames: i32::from_le_bytes(bytes[1064..1068].try_into().unwrap()),
        signon_length: i32::from_le_bytes(bytes[1068..1072].try_into().unwrap()),
    };
    h
}



fn main() {
    let now = Instant::now();
    let bytes = std::fs::read("/home/laiho/Documents/demos/rclonetest/q.dem").unwrap();
    
    let h: Header = parse_header(&bytes);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    //println!("{:?}", h);
    // let sparkle_heart = str::from_utf8(&bytes[1073..107]).unwrap();

}
