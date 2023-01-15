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
    pub fn uid_to_entid(&self, uid: u32) {}
}
