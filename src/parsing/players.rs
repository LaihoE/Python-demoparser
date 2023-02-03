use ahash::{HashMap, HashSet};

use super::parser::JobResult;
use crate::parsing::demo_parsing::*;
use itertools::{Itertools, Unique};

#[derive(Debug, Clone)]
pub struct Players {
    pub players: Vec<UserInfo>,
    pub uid_to_eid: HashMap<u32, Vec<Connection>>,
    pub sid_to_eid: HashMap<u64, Vec<Connection>>,
    pub uid_to_steamid: HashMap<u32, u64>,
    pub uid_to_name: HashMap<u32, String>,
    //pub eid_to_uid: HashMap<u32>
    pub entid_to_uid: HashMap<u32, Vec<ReverseConnection>>,
    pub steamids: HashSet<u64>,
    pub uids: HashSet<u32>,
}
#[derive(Debug, Clone)]
pub struct Connection {
    entid: u32,
    byte: usize,
    tick: i32,
}
#[derive(Debug, Clone)]
pub struct ReverseConnection {
    uid: u32,
    byte: usize,
    tick: i32,
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
        let mut sid_to_eid = HashMap::default();
        let mut eid_to_uid = HashMap::default();
        let mut steamids = HashSet::default();
        let mut uids = HashSet::default();

        for player in &players {
            uid_to_entid
                .entry(player.user_id)
                .or_insert(vec![])
                .push(Connection {
                    entid: player.entity_id,
                    byte: player.byte,
                    tick: player.tick,
                });

            eid_to_uid
                .entry(player.entity_id)
                .or_insert(vec![])
                .push(ReverseConnection {
                    uid: player.user_id,
                    byte: player.byte,
                    tick: player.tick,
                });

            steamids.insert(player.xuid);
            uids.insert(player.user_id);
            uid_to_steamid.insert(player.user_id, player.xuid);
            uid_to_name.insert(player.user_id, player.name.clone());
        }

        Players {
            players: players,
            uid_to_eid: uid_to_entid,
            uid_to_steamid: uid_to_steamid,
            uid_to_name: uid_to_name,
            sid_to_eid: sid_to_eid,
            steamids: steamids,
            uids: uids,
            entid_to_uid: eid_to_uid,
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

    pub fn eid_to_sid(&self, eid: u32, tick: i32) -> Option<u64> {
        match self.entid_to_uid(eid, tick) {
            Some(uid) => self.uid_to_steamid(uid),
            None => {
                //println!("NO SID MAP{:?} {}", eid, tick);
                None
            }
        }
    }

    #[inline(always)]
    pub fn entid_to_uid(&self, eid: u32, tick: i32) -> Option<u32> {
        match self.entid_to_uid.get(&eid) {
            None => {
                return None;
            }
            Some(player_mapping) => {
                for mapping in player_mapping.windows(2) {
                    if mapping[1].tick > tick && mapping[0].tick <= tick {
                        return Some(mapping[0].uid);
                    }
                }
                return Some(player_mapping.last().unwrap().uid);
            }
        }
    }

    pub fn get_steamids(&self) -> Vec<u64> {
        // Unique ids
        self.steamids.iter().map(|x| x.clone()).collect()
    }
    pub fn get_uids(&self) -> Vec<u32> {
        self.uids.iter().map(|x| x.clone()).collect()
    }
}
