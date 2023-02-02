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
            /*
            println!(
                "{} {} {} {}",
                player.entity_id, player.tick, player.xuid, player.user_id
            );
            */
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
            /*
            if player.xuid == 76561198829733633 {
                eid_to_sid
                    .entry(player.entity_id)
                    .or_insert(vec![])
                    .push(ReverseConnection {
                        sid: 76561198829733633,
                        byte: 9999999999,
                        tick: 99999999,
                    });
            }
            */
            //if player.xuid != 0 {
            steamids.insert(player.xuid);
            uids.insert(player.user_id);
            uid_to_steamid.insert(player.user_id, player.xuid);
            uid_to_name.insert(player.user_id, player.name.clone());
            //}
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

    /*
    #[inline(always)]
    pub fn uid_to_entid(&self, uid: u32, tick: i32) -> Option<u32> {
        match self.sid_to_eid.get(&uid) {
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
    */
    #[inline(always)]
    pub fn entid_to_uid(&self, eid: u32, tick: i32) -> Option<u32> {
        match self.entid_to_uid.get(&eid) {
            None => {
                //println!("SID NO MAP{}", eid);
                return None;
            } //panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(player_mapping) => {
                for mapping in player_mapping.windows(2) {
                    if mapping[1].tick > tick && mapping[0].tick <= tick {
                        /*
                        if eid == 21234 && tick < 40000 && tick > 39000 {
                            println!("{} > {}", mapping[1].tick, tick);
                            println!("{:?}", player_mapping);
                            println!("{:?}", mapping[0].uid);
                            println!("{:?}", self.uid_to_steamid(mapping[0].uid));
                        }
                        */
                        /*
                        12 101052 76561198829733633 23
                        2 105534 0 24
                        12 106194 76561198829733633 25

                        */

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
