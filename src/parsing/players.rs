use ahash::{HashMap, HashSet};

use super::parser::JobResult;
use crate::parsing::stringtables::UserInfo;
use itertools::{Itertools, Unique};

#[derive(Debug, Clone)]
pub struct Players {
    pub players: Vec<UserInfo>,
    pub uid_to_eid: HashMap<u32, Vec<Connection>>,
    pub sid_to_eid: HashMap<u64, Vec<Connection>>,
    pub uid_to_steamid: HashMap<u32, u64>,
    pub uid_to_name: HashMap<u32, String>,
    pub entid_to_sid: HashMap<u32, Vec<ReverseConnection>>,
    pub steamids: HashSet<u64>,
}
#[derive(Debug, Clone)]
pub struct Connection {
    entid: u32,
    byte: usize,
    tick: i32,
}
#[derive(Debug, Clone)]
pub struct ReverseConnection {
    sid: u64,
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
        let mut eid_to_sid = HashMap::default();
        let mut steamids = HashSet::default();

        for player in &players {
            uid_to_entid
                .entry(player.user_id)
                .or_insert(vec![])
                .push(Connection {
                    entid: player.entity_id,
                    byte: player.byte,
                    tick: player.tick,
                });
            sid_to_eid
                .entry(player.xuid)
                .or_insert(vec![])
                .push(Connection {
                    entid: player.entity_id,
                    byte: player.byte,
                    tick: player.tick,
                });
            eid_to_sid
                .entry(player.entity_id)
                .or_insert(vec![])
                .push(ReverseConnection {
                    sid: player.xuid,
                    byte: player.byte,
                    tick: player.tick,
                });

            steamids.insert(player.xuid);
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
            entid_to_sid: eid_to_sid,
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
    #[inline(always)]
    pub fn sid_to_entid(&self, sid: u64, tick: i32) -> Option<u32> {
        match self.sid_to_eid.get(&sid) {
            None => None, //panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(player_mapping) => {
                for mapping in player_mapping.windows(2) {
                    //println!("{} {:?} {}", sid, mapping, tick);

                    if mapping[1].tick > tick {
                        //println!("RETURN upper:{}", mapping[0].entid);
                        return Some(mapping[0].entid);
                    }
                }
                //println!("RETURN: {}", player_mapping.last().unwrap().entid);
                return Some(player_mapping.last().unwrap().entid);
            }
        }
    }
    #[inline(always)]
    pub fn entid_to_sid(&self, eid: u32, tick: i32) -> Option<u64> {
        match self.entid_to_sid.get(&eid) {
            None => {
                println!("SID NO MAP{}", eid);
                None
            } //panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(player_mapping) => {
                for mapping in player_mapping.windows(2) {
                    if mapping[1].tick > tick {
                        return Some(mapping[0].sid);
                    }
                }
                return Some(player_mapping.last().unwrap().sid);
            }
        }
    }

    pub fn get_steamids(&self) -> Vec<u64> {
        // Unique ids
        self.steamids.iter().map(|x| x.clone()).collect()
    }
}
