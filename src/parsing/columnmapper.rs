use super::stringtables::UserInfo;
pub use crate::parsing::variants::*;
use ahash::HashMap;
use ahash::HashSet;

#[derive(Debug, Clone)]
pub struct EntColMapper {
    players: HashMap<u32, Vec<EntConnection>>,
    tick_map: HashMap<i32, usize>,
    col_sid_map: HashMap<usize, u64>,
}
#[derive(Debug, Clone)]
pub struct EntConnection {
    steamid: u64,
    tick: i32,
    column: usize,
}

fn ent_col_mapping(players: &Vec<&UserInfo>) -> HashMap<u64, usize> {
    /*
    Maps potentially
    */
    let mut unique_players = HashSet::default();
    for player in players {
        unique_players.insert(player.xuid);
    }
    let mut mapping: HashMap<u64, usize> = HashMap::default();
    for (idx, player) in unique_players.iter().enumerate() {
        mapping.insert(*player, idx);
    }
    mapping
}

impl EntColMapper {
    pub fn new(userinfos: &Vec<&UserInfo>, wanted_ticks: &HashSet<i32>) -> Self {
        let mut tick_map: HashMap<i32, usize> = HashMap::default();
        for (idx, t) in wanted_ticks.iter().enumerate() {
            tick_map.insert((*t).try_into().unwrap(), idx);
        }

        let mut unique_players = HashSet::default();
        for player in userinfos {
            unique_players.insert(player.xuid);
        }
        // Map each steamid to a column idx. No special logic just the order they come in
        let mut sid_to_col_idx = HashMap::default();
        for (idx, player_sid) in unique_players.iter().enumerate() {
            sid_to_col_idx.insert(*player_sid, idx + 1);
        }
        let mut eids: HashMap<u32, Vec<EntConnection>> = HashMap::default();

        for player in userinfos {
            eids.entry(player.entity_id)
                .or_insert(vec![])
                .push(EntConnection {
                    steamid: player.xuid,
                    tick: player.tick,
                    column: *sid_to_col_idx.get(&player.xuid).unwrap_or(&0),
                });
        }
        for (k, v) in &mut eids {
            v.sort_by_key(|x| x.tick);
        }
        let mut col_sid_map = HashMap::default();
        for (k, v) in sid_to_col_idx {
            col_sid_map.insert(v, k);
        }
        EntColMapper {
            players: eids,
            tick_map: tick_map,
            col_sid_map: col_sid_map,
        }
    }
    #[inline(always)]
    fn get_complicated<'a>(
        &self,
        ent_maps_to_these_ids: &'a Vec<EntConnection>,
        entid: u32,
        tick: i32,
    ) -> &'a EntConnection {
        /*
        More complicated one that happens when different players have shared entid
        */
        for connection_idx in 0..ent_maps_to_these_ids.len() - 1 {
            if ent_maps_to_these_ids[connection_idx + 1].tick > tick {
                return &ent_maps_to_these_ids[connection_idx];
            }
        }
        return &ent_maps_to_these_ids[ent_maps_to_these_ids.len() - 1];
    }
    #[inline(always)]
    pub fn get_col(&self, entid: u32, tick: i32) -> usize {
        let ent_maps_to_these_ids = match self.players.get(&entid) {
            None => return 0,
            Some(steamids) => steamids,
        };
        // Most of the time it's this simple (>95%)
        // This entid only maps to one player
        if ent_maps_to_these_ids.len() == 1 {
            return ent_maps_to_these_ids[0].column;
        }
        // Entity id mapped to multiple players :(
        let ent_connection = self.get_complicated(&ent_maps_to_these_ids, entid, tick);
        ent_connection.column
    }

    pub fn get_sid(&self, entid: u32, tick: i32) -> u64 {
        let ent_maps_to_these_ids = match self.players.get(&entid) {
            None => return 0,
            Some(steamids) => steamids,
        };
        // Most of the time it's this simple (>95%)
        // This entid only maps to one player
        if ent_maps_to_these_ids.len() == 1 {
            return ent_maps_to_these_ids[0].steamid;
        }
        // Entity id mapped to multiple players :(
        let ent_connection = self.get_complicated(&ent_maps_to_these_ids, entid, tick);
        ent_connection.steamid
    }
    #[inline(always)]
    pub fn get_tick(&self, tick: i32) -> usize {
        /*
        Returns idx for tick. Mostly interesting for when user only wants some ticks
        */
        return self.tick_map[&tick];
    }
    #[inline(always)]
    pub fn get_col_sid_vec(&self, col: usize, max_ticks: usize) -> Vec<u64> {
        //println!("{:?}", self.col_sid_map);
        //println!("{:?}", col);
        match self.col_sid_map.get(&col) {
            None => vec![0; max_ticks],
            Some(s) => vec![*s; max_ticks],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::columnmapper::EntColMapper;
    use crate::parsing::columnmapper::EntConnection;
    use ahash::HashMap;

    pub fn init_mapper() -> EntColMapper {
        let mut eids: HashMap<u32, Vec<EntConnection>> = HashMap::default();

        let ethree = [
            EntConnection {
                steamid: 111,
                tick: -18298,
                column: 2,
            },
            EntConnection {
                steamid: 222,
                tick: 79663,
                column: 3,
            },
            EntConnection {
                steamid: 111,
                tick: 283533,
                column: 2,
            },
            EntConnection {
                steamid: 222,
                tick: 318699,
                column: 3,
            },
        ];
        eids.insert(3, ethree.to_vec());

        EntColMapper {
            players: eids,
            tick_map: HashMap::default(),
            col_sid_map: HashMap::default(),
        }
    }

    #[test]
    fn middle_sid_ok() {
        let tick = 85000;
        let entid = 3;
        let ecm = init_mapper();

        let result = ecm.get_sid(entid, tick);
        assert_eq!(result, 222);
    }
    #[test]
    fn last_sid_ok() {
        let tick = 9999999;
        let entid = 3;
        let ecm = init_mapper();

        let result = ecm.get_sid(entid, tick);
        assert_eq!(result, 222);
    }

    #[test]
    fn fist_sid_ok() {
        let tick = -99999;
        let entid = 3;
        let ecm = init_mapper();

        let result = ecm.get_sid(entid, tick);
        assert_eq!(result, 111);
    }
}
