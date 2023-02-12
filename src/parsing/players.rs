use super::parser::JobResult;
use crate::parsing::demo_parsing::*;
use ahash::{HashMap, HashSet};
use itertools::{Itertools, Unique};

#[derive(Debug, Clone)]
pub struct Players {
    pub players: Vec<UserInfo>,
    pub uid_to_eid: HashMap<u32, Vec<Connection>>,
    pub sid_to_entid: HashMap<u64, Vec<Connection>>,
    pub uid_to_steamid: HashMap<u32, u64>,
    pub uid_to_name: HashMap<u32, String>,
    //pub eid_to_uid: HashMap<u32>
    pub entid_to_uid: HashMap<u32, Vec<ReverseConnection>>,
    pub steamids: HashSet<u64>,
    pub uids: HashSet<u32>,
    pub is_easy: Vec<bool>,
    pub eid_sid_easy: Vec<u64>,
    pub is_easy_uid: Vec<bool>,
    pub sid_to_name: HashMap<u64, String>,
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
        let mut sid_to_entid = HashMap::default();
        let mut uid_to_entid = HashMap::default();
        let mut uid_to_steamid = HashMap::default();
        let mut uid_to_name = HashMap::default();
        let mut eid_to_uid = HashMap::default();
        let mut steamids = HashSet::default();
        let mut uids = HashSet::default();
        let mut is_easy = vec![false; 128];
        let mut is_easy_uid = vec![true; 1024];

        let mut eid_to_sid_easy = vec![0; 128];
        let mut eid_to_sid_simple = HashMap::default();

        let mut sid_to_name = HashMap::default();

        let mut overlap = HashMap::default();
        let mut overlap_uid = HashMap::default();

        for player in &players {
            // Now changing name wont affect, maybe dont want this?
            sid_to_name.insert(player.xuid, player.name.clone());

            overlap
                .entry(player.entity_id)
                .or_insert(HashSet::default())
                .insert(player.user_id);

            overlap_uid
                .entry(player.user_id)
                .or_insert(HashSet::default())
                .insert(player.entity_id);

            uid_to_entid
                .entry(player.user_id)
                .or_insert(vec![])
                .push(Connection {
                    entid: player.entity_id,
                    byte: player.byte,
                    tick: player.tick,
                });
            sid_to_entid
                .entry(player.xuid)
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
            eid_to_sid_simple.insert(player.entity_id, player.xuid);

            steamids.insert(player.xuid);
            uids.insert(player.user_id);
            uid_to_steamid.insert(player.user_id, player.xuid);
            uid_to_name.insert(player.user_id, player.name.clone());
        }

        for (k, v) in &overlap {
            if v.len() == 1 {
                is_easy[*k as usize] = true;
                eid_to_sid_easy[*k as usize] = eid_to_sid_simple[&k];
            } else {
                is_easy[*k as usize] = true;
            }
        }
        for (k, v) in overlap {
            if v.len() > 1 {
                for entid in v {
                    is_easy_uid[entid as usize] = false;
                }
            }
        }

        Players {
            players: players,
            sid_to_name: sid_to_name,
            uid_to_eid: uid_to_entid,
            uid_to_steamid: uid_to_steamid,
            uid_to_name: uid_to_name,
            sid_to_entid: sid_to_entid,
            steamids: steamids,
            uids: uids,
            entid_to_uid: eid_to_uid,
            is_easy: is_easy,
            eid_sid_easy: eid_to_sid_easy,
            is_easy_uid: is_easy_uid,
        }
    }
    pub fn steamid_to_name(&self, sid: u64) -> String {
        self.sid_to_name[&sid].clone()
    }

    #[inline(always)]
    pub fn sid_to_entid(&self, sid: u64, tick: i32) -> Option<u32> {
        match self.sid_to_entid.get(&sid) {
            None => {
                return None;
            }
            Some(player_mapping) => {
                for mapping in player_mapping.windows(2) {
                    if mapping[1].tick > tick && mapping[0].tick <= tick {
                        return Some(mapping[0].entid);
                    }
                }
                return Some(player_mapping.last().unwrap().entid);
            }
        }
    }
    pub fn uid_to_entid_tick(&self, uid: u32, tick: i32) -> Option<u32> {
        match self.uid_to_eid.get(&uid) {
            None => None, //panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(player_mapping) => {
                for mapping in player_mapping.windows(2) {
                    if mapping[1].tick > tick && mapping[0].tick <= tick {
                        return Some(mapping[0].entid);
                    }
                }
                return Some(player_mapping.last().unwrap().entid);
            }
        }
    }
    pub fn uid_to_entid_byte(&self, uid: u32, byte: usize) -> Option<u32> {
        match self.uid_to_eid.get(&uid) {
            None => None, //panic!("NO USERID MAPPING TO ENTID: {}", uid),
            Some(player_mapping) => {
                for mapping in player_mapping.windows(2) {
                    if mapping[1].byte > byte && mapping[0].byte <= byte {
                        return Some(mapping[0].entid);
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
    pub fn eid_to_sid(&self, eid: u32, tick: i32) -> Option<u64> {
        match self.entid_to_uid(eid, tick) {
            Some(uid) => self.uid_to_steamid(uid),
            None => None,
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
