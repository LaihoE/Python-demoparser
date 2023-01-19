use ahash::HashMap;

use crate::parsing::stringtables::UserInfo;

use super::parser::JobResult;

#[derive(Debug, Clone)]
pub struct Players {
    pub players: Vec<UserInfo>,
    pub uid_to_eid: HashMap<u32, Vec<Connection>>,
}
#[derive(Debug, Clone)]
pub struct Connection {
    entid: u32,
    byte: usize,
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
        let mut uid_to_entid = HashMap::default();

        for player in &players {
            uid_to_entid
                .entry(player.user_id)
                .or_insert(vec![])
                .push(Connection {
                    entid: player.entity_id,
                    byte: player.byte,
                });
        }
        for (k, v) in &uid_to_entid {
            //println!("{} {:?}", k, v);
        }
        Players {
            players: players,
            uid_to_eid: uid_to_entid,
        }
    }
    pub fn uid_to_entid(&self, uid: u32, byte: usize) -> u32 {
        match self.uid_to_eid.get(&uid) {
            None => panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(player_mapping) => {
                for mapping in player_mapping {
                    if mapping.byte > byte {
                        return mapping.entid;
                    }
                }
                return player_mapping.last().unwrap().entid;
            }
        }
    }
}
