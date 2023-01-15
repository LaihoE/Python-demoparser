use std::time::Instant;

use crate::parsing::parser;
use serde::Deserialize;
pub struct Cache {
    pub deltas: Vec<Delta>,
    pub game_events: Vec<GameEventIdx>,
    pub stringtables: Vec<Stringtables>,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    byte: u64,
    idx: u32,
    entid: u32,
}
#[derive(Debug, Deserialize)]
pub struct GameEventIdx {
    byte: u64,
    id: i32,
}
#[derive(Debug, Deserialize)]
pub struct Stringtables {
    byte: u64,
}

impl Cache {
    pub fn set_deltas(&mut self) {
        let mut rdr = csv::Reader::from_path(
            "/home/laiho/Documents/programming/rust/recent/Python-demoparser/foo.csv",
        )
        .unwrap();
        for result in rdr.deserialize() {
            let record: Delta = result.unwrap();
            self.deltas.push(record);
        }
    }
    pub fn set_game_events(&mut self) {
        let mut rdr = csv::Reader::from_path(
            "/home/laiho/Documents/programming/rust/recent/Python-demoparser/events.csv",
        )
        .unwrap();
        for result in rdr.deserialize() {
            let record: GameEventIdx = result.unwrap();
            self.game_events.push(record);
        }
    }
    pub fn set_stringtables(&mut self) {
        let mut rdr = csv::Reader::from_path(
            "/home/laiho/Documents/programming/rust/recent/Python-demoparser/st.csv",
        )
        .unwrap();
        for result in rdr.deserialize() {
            let record: Stringtables = result.unwrap();
            self.stringtables.push(record);
        }
    }
    pub fn get_stringtables(&self) -> Vec<u64> {
        self.stringtables.iter().map(|s| s.byte).collect()
    }
    pub fn get_event_by_id(&self, wanted_id: i32) -> Vec<u64> {
        let mut kills_idx = 0;
        let before = Instant::now();
        for i in 0..self.deltas.len() {
            let byte = self.deltas[i].byte;
            //println!("{} < {}", self.game_events[kills_idx].byte, byte);
            if self.game_events[kills_idx].byte < byte {
                let byte_want =
                    self.find_last_val(&self.deltas, &self.game_events[kills_idx], i, 20);
                //println!("BW {:?}", byte_want);
                kills_idx += 1;
                if kills_idx == self.game_events.len() {
                    break;
                }
            }
        }
        println!("{:2?}", before.elapsed());
        let mut v = vec![];
        for ev in &self.game_events {
            if ev.id == wanted_id {
                v.push(ev.byte);
            }
        }
        v
    }
    /*
    let v = get_v();
    let kills = vec![(1000000, 5), (2000000, 4), (3000000, 5)];
    let wanted_idx = 20;
    let mut kills_idx = 0;

    let before = Instant::now();

    for i in 0..v.len() {
        let byte = v[i].byte;
        if kills[kills_idx].0 < byte {
            let byte_want = find_last_val(&v, &kills[kills_idx], i, wanted_idx);
            println!("{:?}", byte_want);
            kills_idx += 1;
            if kills_idx == kills.len() {
                break;
            }
        }
    }
    */

    pub fn find_last_val(
        &self,
        v: &Vec<Delta>,
        kill: &GameEventIdx,
        i: usize,
        wanted_idx: u32,
    ) -> u64 {
        /*
        Find most recent delta
        */
        for i in (0..i).rev() {
            if v[i].idx == wanted_idx {
                return v[i].byte;
            }
        }
        panic!("no found")
    }
}
