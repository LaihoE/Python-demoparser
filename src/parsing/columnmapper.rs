use super::game_events::KeyData;
use super::parser_settings::Maps;
use super::stringtables::UserInfo;
pub use crate::parsing::variants::*;
pub use crate::parsing::variants::*;
use ahash::HashMap;
use ahash::HashSet;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct EntColMapper {
    pub players: HashMap<u32, Vec<EntConnection>>,
    pub tick_map: HashMap<i32, usize>,
    pub col_sid_map: HashMap<usize, u64>,
    pub uid_to_entid: HashMap<u32, Vec<u32>>,
    pub idx_pos: HashMap<i32, usize>,
    pub col_idx: HashMap<usize, i32>,
}

#[derive(Debug, Clone)]
pub struct EntConnection {
    steamid: u64,
    tick: i32,
    column: usize,
}

fn ent_col_mapping(players: &Vec<&UserInfo>) -> HashMap<u64, usize> {
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
    pub fn new(
        userinfos: &Vec<&UserInfo>,
        wanted_ticks: &HashSet<i32>,
        wanted_props: &Vec<String>,
        max_ticks: usize,
        parser_maps: &Maps,
    ) -> Self {
        let mut tick_map: HashMap<i32, usize> = HashMap::default();
        for (idx, t) in wanted_ticks.iter().enumerate() {
            tick_map.insert((*t).try_into().unwrap(), idx);
        }

        let mut real_props = rm_user_friendly_names(wanted_props);
        let int_props = str_props_to_int_props(&real_props, &parser_maps);

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
        // println!("X {:?}", sid_to_col_idx);

        for player in userinfos {
            // println!("{:?} {} {}", player.user_id, player.name, player.tick);
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
        let mut uid_to_entid: HashMap<u32, Vec<u32>> = HashMap::default();

        for player in userinfos {
            uid_to_entid
                .entry(player.user_id)
                .or_insert(vec![])
                .push(player.entity_id);
        }

        let (col_mapping, idx_col) = create_idx_col_mapping(&int_props);

        EntColMapper {
            players: eids,
            tick_map: tick_map,
            col_sid_map: col_sid_map,
            uid_to_entid: uid_to_entid,
            col_idx: idx_col,
            idx_pos: col_mapping,
        }
    }

    pub fn entid_from_uid(&self, uid: &KeyData) -> u32 {
        /*
        Map uid -> entid. uid seems unique for players.
        */
        match uid {
            KeyData::Short(uid) => match self.uid_to_entid.get(&(*uid as u32)) {
                Some(eid) => return eid[0],
                None => panic!("No entity id found for user id: {}", uid),
            },
            _ => panic!("user id should be KeyData::Short. Got: {:?}", uid),
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
    pub fn get_player_col(&self, entid: u32, tick: i32) -> usize {
        let ent_maps_to_these_ids = match self.players.get(&entid) {
            None => {
                return 0;
            }
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
    #[inline(always)]
    pub fn get_prop_col(&self, prop_idx: &i32) -> usize {
        return self.idx_pos[&(*prop_idx)].try_into().unwrap();
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
        match self.col_sid_map.get(&col) {
            None => {
                vec![0; max_ticks]
            }
            Some(s) => vec![*s; max_ticks],
        }
    }
}

fn create_idx_col_mapping(prop_indicies: &Vec<i32>) -> (HashMap<i32, usize>, HashMap<usize, i32>) {
    /*
    Create mapping from property index into column index.
    This is needed because prop indicies might be:
    24, 248, 354 and we can't be creating 354 columns so
    we map it into 0,1,2..
    */
    let mut idx_pos: HashMap<i32, usize> = HashMap::default();
    let mut col_idx: HashMap<usize, i32> = HashMap::default();

    for (cnt, p_idx) in prop_indicies.iter().enumerate() {
        idx_pos.insert(*p_idx, cnt);
        col_idx.insert(cnt, *p_idx);
    }
    (idx_pos, col_idx)
}
fn col_str_mapping(col_indicies: &Vec<i32>, parser_maps: &Maps) -> Vec<String> {
    /*
    Maps column index to it's human readable name
    */
    let serverclass_map = &parser_maps.serverclass_map;
    let props = &serverclass_map.get(&40).unwrap().props;
    let mut str_names = vec![];

    for ci in col_indicies {
        let s = &props[*ci as usize];
        str_names.push(s.name.to_owned());
    }
    str_names
}

// String_name --> Propidx --> Colidx

struct PropIdentity {
    str_name: String,
    prop_idx: i32,
    col_idx: i32,
}

pub fn str_props_to_int_props(str_props: &Vec<String>, parser_maps: &Maps) -> Vec<i32> {
    let mut int_props = vec![];

    for s in str_props {
        //if s != "m_vecOrigin" {
        int_props.push(str_prop_to_int(s, &parser_maps));
        //}
    }
    int_props
}

fn str_prop_to_int(wanted_prop: &str, parser_maps: &Maps) -> i32 {
    /*
    Maps string names to their prop index (used in packet ents)
    For example m_angEyeAngles[0] --> 20
    Mainly need to pay attention to manager props comming from different
    serverclass
    */
    match wanted_prop {
        "m_vecOrigin_X" => return 10000i32,
        "m_vecOrigin_Y" => return 10001i32,
        _ => {}
    }

    let serverclass_map = &parser_maps.serverclass_map;
    let sv_cls = &serverclass_map.get(&40).unwrap();

    match sv_cls.props.iter().position(|x| x.name == wanted_prop) {
        Some(idx) => idx as i32,
        None => panic!("Could not find prop idx for {}", wanted_prop),
    }
}
pub fn rm_user_friendly_names(names: &Vec<String>) -> Vec<String> {
    let mut unfriendly_names = vec![];
    for name in names {
        match &name[..] {
            "X" => unfriendly_names.push("m_vecOrigin_X".to_string()),
            "Y" => unfriendly_names.push("m_vecOrigin_Y".to_string()),
            "Z" => unfriendly_names.push("m_vecOrigin[2]".to_string()),
            "ammo" => unfriendly_names.push("m_iClip1".to_string()),
            "velocity_X" => unfriendly_names.push("m_vecVelocity[0]".to_string()),
            "velocity_Y" => unfriendly_names.push("m_vecVelocity[1]".to_string()),
            "velocity_Z" => unfriendly_names.push("m_vecVelocity[2]".to_string()),
            "viewangle_pitch" => unfriendly_names.push("m_angEyeAngles[0]".to_string()),
            "viewangle_yaw" => unfriendly_names.push("m_angEyeAngles[1]".to_string()),
            "ducked" => unfriendly_names.push("m_bDucked".to_string()),
            "in_buy_zone" => unfriendly_names.push("m_bInBuyZone".to_string()),
            "scoped" => unfriendly_names.push("m_bIsScoped".to_string()),
            "health" => unfriendly_names.push("m_iHealth".to_string()),
            "flash_duration" => unfriendly_names.push("m_flFlashDuration".to_string()),
            "aimpunch_X" => unfriendly_names.push("m_aimPunchAngle_X".to_string()),
            "aimpunch_Y" => unfriendly_names.push("m_aimPunchAngle_Y".to_string()),
            "aimpunch_Z" => unfriendly_names.push("m_aimPunchAngle_Z".to_string()),
            "aimpunch_vel_X" => unfriendly_names.push("m_aimPunchAngleVel_X".to_string()),
            "aimpunch_vel_Y" => unfriendly_names.push("m_aimPunchAngleVel_Y".to_string()),
            "aimpunch_vel_Z" => unfriendly_names.push("m_aimPunchAngleVel_Z".to_string()),
            "balance" => unfriendly_names.push("m_iAccount".to_string()),
            "ping" => unfriendly_names.push("m_iPing".to_string()),
            "score" => unfriendly_names.push("m_iScore".to_string()),
            "deaths" => unfriendly_names.push("m_iDeaths".to_string()),
            "kills" => unfriendly_names.push("m_iKills".to_string()),
            "assists" => unfriendly_names.push("m_iAssists".to_string()),
            "mvps" => unfriendly_names.push("m_iMVPs".to_string()),
            "armor" => unfriendly_names.push("m_iArmor".to_string()),
            "silencer_on" => unfriendly_names.push("m_bSilencerOn".to_string()),
            "place_name" => unfriendly_names.push("m_szLastPlaceName".to_string()),
            "total_enemies_flashed" => {
                unfriendly_names.push("m_iMatchStats_EnemiesFlashed_Total".to_string())
            }
            "total_util_damage" => {
                unfriendly_names.push("m_iMatchStats_UtilityDamage_Total".to_string())
            }
            "total_cash_earned" => {
                unfriendly_names.push("m_iMatchStats_CashEarned_Total".to_string())
            }
            "total_objective_total" => {
                unfriendly_names.push("m_iMatchStats_Objective_Total".to_string())
            }
            "total_headshots" => {
                unfriendly_names.push("m_iMatchStats_HeadShotKills_Total".to_string())
            }
            "total_assists" => unfriendly_names.push("m_iMatchStats_Assists_Total".to_string()),
            "total_deaths" => unfriendly_names.push("m_iMatchStats_Deaths_Total".to_string()),
            "total_live_time" => unfriendly_names.push("m_iMatchStats_LiveTime_Total".to_string()),
            "total_kill_reward" => {
                unfriendly_names.push("m_iMatchStats_KillReward_Total".to_string())
            }
            "total_equipment_value" => {
                unfriendly_names.push("m_iMatchStats_EquipmentValue_Total".to_string())
            }
            "total_damage" => unfriendly_names.push("m_iMatchStats_Damage_Total".to_string()),
            "3ks" => unfriendly_names.push("m_iMatchStats_3k_Total".to_string()),
            "4ks" => unfriendly_names.push("m_iMatchStats_4k_Total".to_string()),
            "5ks" => unfriendly_names.push("m_iMatchStats_5k_Total".to_string()),
            "total_kills" => unfriendly_names.push("m_iMatchStats_Kills_Total".to_string()),
            "is_auto_muted" => unfriendly_names.push("m_bHasCommunicationAbuseMute".to_string()),
            "friendly_honors" => {
                unfriendly_names.push("m_nPersonaDataPublicCommendsFriendly".to_string())
            }
            "teacher_honors" => {
                unfriendly_names.push("m_nPersonaDataPublicCommendsTeacher".to_string())
            }
            "leader_honors" => {
                unfriendly_names.push("m_nPersonaDataPublicCommendsLeader".to_string())
            }
            "public_level" => unfriendly_names.push("m_nPersonaDataPublicLevel".to_string()),
            "active_coin_rank" => unfriendly_names.push("m_nActiveCoinRank".to_string()),
            "cash_spent_this_round" => unfriendly_names.push("m_iCashSpentThisRound".to_string()),
            "total_cash_spent" => unfriendly_names.push("m_iTotalCashSpent".to_string()),
            "controlled_by_player" => unfriendly_names.push("m_iControlledByPlayer".to_string()),
            "controlled_player" => unfriendly_names.push("m_iControlledPlayer".to_string()),
            "controlling_bot" => unfriendly_names.push("m_bControllingBot".to_string()),
            "lifetime_start" => unfriendly_names.push("m_iLifetimeStart".to_string()),
            "lifetime_end" => unfriendly_names.push("m_iLifetimeEnd".to_string()),
            "connected" => unfriendly_names.push("m_bConnected".to_string()),
            "holding_look_weapon" => unfriendly_names.push("m_bIsHoldingLookAtWeapon".to_string()),
            "looking_at_weapon" => unfriendly_names.push("m_bIsLookingAtWeapon".to_string()),
            "headshots_this_round" => {
                unfriendly_names.push("m_iNumRoundKillsHeadshots".to_string())
            }
            "concurrent_killed" => unfriendly_names.push("m_nLastConcurrentKilled".to_string()),
            "freeze_end_eq_val" => {
                unfriendly_names.push("m_unFreezetimeEndEquipmentValue".to_string())
            }
            "round_start_eq_val" => {
                unfriendly_names.push("m_unRoundStartEquipmentValue".to_string())
            }
            "equipment_value" => unfriendly_names.push("m_unCurrentEquipmentValue".to_string()),
            "flash_alpha" => unfriendly_names.push("m_flFlashMaxAlpha".to_string()),
            "has_helmet" => unfriendly_names.push("m_bHasHelmet".to_string()),
            "has_heavy_armor" => unfriendly_names.push("m_bHasHeavyArmor".to_string()),
            "detected_enemy_sensor" => {
                unfriendly_names.push("m_flDetectedByEnemySensorTime".to_string())
            }
            "is_rescuing" => unfriendly_names.push("m_bIsRescuing".to_string()),
            "molotov_dmg_time" => unfriendly_names.push("m_fMolotovDamageTime".to_string()),
            "molotov_use_time" => unfriendly_names.push("m_fMolotovUseTime".to_string()),
            "moved_since_spawn" => unfriendly_names.push("m_bHasMovedSinceSpawn".to_string()),
            "resume_zoom" => unfriendly_names.push("m_bResumeZoom".to_string()),
            "is_walking" => unfriendly_names.push("m_bIsWalking".to_string()),
            "is_defusing" => unfriendly_names.push("m_bIsDefusing".to_string()),
            "has_defuser" => unfriendly_names.push("m_bHasDefuser".to_string()),
            "in_bomb_zone" => unfriendly_names.push("m_bInBombZone".to_string()),
            "granade_counter" => unfriendly_names.push("m_iThrowGrenadeCounter".to_string()),
            "last_made_noise_time" => unfriendly_names.push("m_flLastMadeNoiseTime".to_string()),
            "spotted" => unfriendly_names.push("m_bSpotted".to_string()),
            "elasticity" => unfriendly_names.push("m_flElasticity".to_string()),
            "team_num" => unfriendly_names.push("m_iTeamNum".to_string()),
            "velocity_modifier" => unfriendly_names.push("m_flVelocityModifier".to_string()),
            "next_think_tick" => unfriendly_names.push("m_nNextThinkTick".to_string()),
            "friction" => unfriendly_names.push("m_flFriction".to_string()),
            "on_target" => unfriendly_names.push("m_fOnTarget".to_string()),
            "vec_view_offset0" => unfriendly_names.push("m_vecViewOffset[0]".to_string()),
            "vec_view_offset1" => unfriendly_names.push("m_vecViewOffset[1]".to_string()),
            "is_wearing_suit" => unfriendly_names.push("m_bWearingSuit".to_string()),
            "jump_time_msecs" => unfriendly_names.push("m_nJumpTimeMsecs".to_string()),
            "duck_time_msecs" => unfriendly_names.push("m_nDuckJumpTimeMsecs".to_string()),
            "in_duck_jump" => unfriendly_names.push("m_bInDuckJump".to_string()),
            "last_duck_time" => unfriendly_names.push("m_flLastDuckTime".to_string()),
            "is_ducking" => unfriendly_names.push("m_bDucking".to_string()),

            _ => unfriendly_names.push(name.to_string()),
        }
    }
    unfriendly_names
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
            uid_to_entid: HashMap::default(),
            col_idx: HashMap::default(),
            idx_pos: HashMap::default(),
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
