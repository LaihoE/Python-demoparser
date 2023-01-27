use ahash::HashMap;

use crate::parsing::stringtables::UserInfo;

use super::parser::JobResult;

#[derive(Debug, Clone)]
pub struct Players {
    pub players: Vec<UserInfo>,
    pub uid_to_eid: HashMap<u32, Vec<Connection>>,
    pub uid_to_steamid: HashMap<u32, u64>,
    pub uid_to_name: HashMap<u32, String>,
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
        let mut uid_to_steamid = HashMap::default();
        let mut uid_to_name = HashMap::default();

        for player in &players {
            //println!("{} {}", player.entity_id, player.xuid);
            uid_to_entid
                .entry(player.user_id)
                .or_insert(vec![])
                .push(Connection {
                    entid: player.entity_id,
                    byte: player.byte,
                });
            uid_to_steamid.insert(player.user_id, player.xuid);
            uid_to_name.insert(player.user_id, player.name.clone());
        }
        for (k, v) in &uid_to_entid {
            // println!("{} {:?}", k, v);
        }
        Players {
            players: players,
            uid_to_eid: uid_to_entid,
            uid_to_steamid: uid_to_steamid,
            uid_to_name: uid_to_name,
        }
    }
    pub fn uid_to_entid(&self, uid: u32, byte: usize) -> Option<u32> {
        match self.uid_to_eid.get(&uid) {
            None => None, //panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(player_mapping) => {
                for mapping in player_mapping {
                    if mapping.byte > byte {
                        return Some(mapping.entid);
                    }
                }
                return Some(player_mapping.last().unwrap().entid);
            }
        }
    }
    pub fn uid_to_steamid(&self, uid: u32) -> Option<u64> {
        match self.uid_to_steamid.get(&uid) {
            None => None, //panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(sid) => {
                return Some(*sid);
            }
        }
    }
    pub fn uid_to_name(&self, uid: u32) -> Option<String> {
        match self.uid_to_name.get(&uid) {
            None => None, //panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(sid) => {
                return Some(sid.to_owned());
            }
        }
    }
}
