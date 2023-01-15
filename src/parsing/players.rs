use crate::parsing::stringtables::UserInfo;

use super::parser::JobResult;

pub struct Players {
    pub players: Vec<UserInfo>,
}

impl Players {
    pub fn new(v: &Vec<JobResult>) -> Self {
        let mut players = vec![];
        // Takes list of jobresults and filters players from it
        for job in v {
            match job {
                JobResult::StringTables(st) => {
                    for player in st {
                        players.push(player.clone());
                    }
                }
                _ => {}
            }
        }
        Players { players: players }
    }
    pub fn uid_to_entid(&self, uid: i16, tick: i32) -> u32 {
        for player in &self.players {
            if player.user_id == uid.try_into().unwrap() {
                return player.entity_id;
            }
        }
        panic!("No uid found")
    }
}
