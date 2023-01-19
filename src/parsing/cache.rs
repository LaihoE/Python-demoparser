use std::time::Instant;

use crate::parsing::parser;
use serde::Deserialize;

use super::{parser::JobResult, players::Players};
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
            if record.id == 24 {
                self.game_events.push(record);
            }
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
    pub fn get_event_by_id(&self, wanted_id: i32) -> Vec<&GameEventIdx> {
        self.game_events
            .iter()
            .filter(|x| x.id == wanted_id)
            .collect()
    }
    pub fn get_event_bytes_by_id(&self, wanted_id: i32) -> Vec<u64> {
        self.game_events
            .iter()
            .filter(|x| x.id == wanted_id)
            .map(|x| x.byte)
            .collect()
    }

    pub fn set_game_event_jobs(&mut self, game_evs: Vec<&JobResult>) {
        let mut v = vec![];
        for event in game_evs {
            match event {
                JobResult::GameEvents(ge) => v.push(ge[0]),
                _ => {}
            }
        }
    }

    pub fn get_event_deltas(&self, wanted_id: u32, players: &Players) -> Vec<u64> {
        let mut v = vec![];
        let wanted_events = self.get_event_by_id(wanted_id as i32);

        let mut kills_idx = 0;
        for i in 0..self.deltas.len() {
            let delta_start_byte = self.deltas[i].byte;

            if wanted_events[kills_idx].byte < delta_start_byte {
                let byte_want = self.find_last_val(
                    &self.deltas,
                    &wanted_events[kills_idx],
                    i,
                    wanted_id,
                    players,
                    self.game_events[kills_idx].byte,
                );
                v.push(byte_want);
                //println!("GE: {:?} {byte_want}", self.game_events);

                kills_idx += 1;
                if kills_idx == wanted_events.len() {
                    break;
                }
            }
        }
        println!("Number bytes {:?}", v.len());
        v
    }

    pub fn find_last_val(
        &self,
        v: &Vec<Delta>,
        kill: &GameEventIdx,
        i: usize,
        wanted_idx: u32,
        players: &Players,
        byte: u64,
    ) -> u64 {
        /*
        Find most recent delta
        */
        for i in (0..i).rev() {
            if v[i].idx == wanted_idx
                && v[i].entid == players.uid_to_entid(84 as u32, byte as usize)
            {
                return v[i].byte;
            }
        }
        panic!("no found")
    }
}
