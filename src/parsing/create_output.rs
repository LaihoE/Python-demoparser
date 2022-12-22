use super::data_table::ServerClasses;
use super::entities::PacketEntsOutput;
use super::game_events::GameEvent;
use super::stringtables::StringTable;
use super::stringtables::UserInfo;
use crate::parsing::columnmapper::EntColMapper;
use crate::parsing::data_table::ServerClass;
use crate::parsing::game_events;
use crate::parsing::parser::JobResult;
use crate::parsing::parser::ParsingMaps;
pub use crate::parsing::variants::*;
use crate::Parser;
use ahash::HashMap;
use ahash::HashSet;
use ndarray::s;
use ndarray::ArrayBase;
use ndarray::Dim;
use ndarray::OwnedRepr;
use polars::frame::DataFrame;
use polars::prelude::*;
use polars::series::Series;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

// Todo entid when disconnected
fn eid_sid_stack(eid: u32, max_ticks: i32, players: &Vec<&UserInfo>) -> Vec<u64> {
    let mut eids: HashMap<u32, Vec<(u64, i32)>> = HashMap::default();
    for player in players {
        eids.entry(player.entity_id)
            .or_insert(vec![])
            .push((player.xuid, player.tick));
    }

    let steamids = match eids.get(&eid) {
        None => return vec![0; max_ticks as usize],
        Some(steamids) => steamids,
    };
    // Most of the time it's this simple (>95%)
    if steamids.len() == 1 {
        return vec![steamids[0].0; max_ticks as usize];
    }
    // But can also be this complicated when entids map to different players
    let mut ticks = vec![];
    let mut player_idx = 0;
    for tick in 0..max_ticks {
        if tick < steamids[player_idx].1 {
            ticks.push(steamids[player_idx].0);
        } else {
            if player_idx == steamids.len() - 1 {
                ticks.push(steamids[player_idx].0);
            } else {
                ticks.push(steamids[player_idx].0);
                player_idx += 1;
            }
        }
    }
    ticks
}

pub fn filter_jobresults(
    jobs: &Vec<JobResult>,
) -> (Vec<&PacketEntsOutput>, Vec<&GameEvent>, Vec<&UserInfo>) {
    /*
    Groups jobresults by their type
    */
    let before = Instant::now();
    let mut packet_ents = vec![];
    let mut game_events = vec![];
    let mut stringtables = vec![];

    for j in jobs {
        match j {
            JobResult::PacketEntities(p) => match p {
                Some(p) => packet_ents.push(p),
                None => {}
            },
            JobResult::GameEvents(ge) => {
                game_events.extend(ge);
            }
            JobResult::StringTables(st) => {
                stringtables.extend(st);
            }
            _ => {}
        }
    }
    (packet_ents, game_events, stringtables)
}

impl Parser {
    pub fn insert_props_into_df(
        &self,
        packet_ents: Vec<&PacketEntsOutput>,
        max_ticks: usize,
        int_props: &Vec<i32>,
        df: &mut ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
        ecm: &EntColMapper,
    ) -> HashMap<usize, i32> {
        let before = Instant::now();

        // Map prop idx into its column.
        let (col_mapping, idx_col) = create_idx_col_mapping(&int_props);
        // For every packetEnt message during game
        for packet_ent_msg in packet_ents {
            // For every entity in the message
            for single_ent in &packet_ent_msg.data {
                // For every updated value for the entity
                for prop in single_ent {
                    match prop.data {
                        PropData::F32(f) => {
                            println!("{:?}", prop);
                            let prop_col = col_mapping[&prop.prop_inx];
                            let player_col = ecm.get_col(prop.ent_id as u32, packet_ent_msg.tick);
                            df[[player_col, prop_col, packet_ent_msg.tick as usize]] = f as f32;
                        }
                        PropData::I32(i) => {
                            let prop_col = col_mapping[&prop.prop_inx];
                            let player_col = ecm.get_col(prop.ent_id as u32, packet_ent_msg.tick);
                            //let tick = ecm.get_tick(packet_ent_msg.tick);
                            df[[player_col, prop_col, packet_ent_msg.tick as usize]] = i as f32;
                        }
                        // Todo string columns
                        _ => {}
                    }
                }
            }
        }
        println!("X {:2?}", before.elapsed());
        idx_col
    }

    pub fn get_raw_df(
        &mut self,
        jobs: &Vec<JobResult>,
        parser_maps: Arc<RwLock<ParsingMaps>>,
        df: &mut ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
        max_ticks: usize,
    ) -> Vec<Series> {
        // Group jobs by type
        let (packet_ents, game_events, stringtables) = filter_jobresults(jobs);
        let ecm = EntColMapper::new(&stringtables, &self.settings.wanted_ticks);

        let mut real_props = rm_user_friendly_names(&self.settings.wanted_props);
        println!("REAL PROSP {:?}", real_props);

        // println!("{:?}", self.settings.wanted_props);
        // let ent_mapping = ent_col_mapping(&stringtables);

        let ticks: Vec<i32> = (0..max_ticks).into_iter().map(|t| t as i32).collect();
        let int_props = str_props_to_int_props(&real_props, parser_maps.clone());

        let str_names = self.insert_props_into_df(packet_ents, max_ticks, &int_props, df, &ecm);

        let mut series_players = vec![];
        let mut ticks_col: Vec<i32> = vec![];
        let mut steamids_col: Vec<u64> = vec![];
        let before = Instant::now();

        for (propcol, prop_name) in str_names.iter().enumerate() {
            let mut this_prop_col: Vec<f32> = Vec::with_capacity(10 * max_ticks);

            for entid in 1..10 {
                // Metadata
                if propcol == 0 {
                    ticks_col.extend(&ticks);
                    steamids_col.extend(ecm.get_col_sid_vec(entid, max_ticks));
                }
                // Props
                let v = &df.slice(s![entid, propcol, ..]);
                this_prop_col.extend(&df.slice(s![entid, propcol, ..]));
            }
            let n = str_names[prop_name.0];
            let props = Series::new(&n.to_string(), &this_prop_col);
            series_players.push(props);
        }
        let steamids = Series::new("steamids", steamids_col);
        let ticks = Series::new("ticks", ticks_col);

        series_players.push(steamids);
        series_players.push(ticks);

        println!("{:2?}", before.elapsed());

        series_players
    }
}
/*
#[inline(always)]
pub fn fill_none_with_most_recent(v: &mut [f32]) {
    /*
    Called ffil in pandas, not sure about polars.

    For example:
    Input: Vec![1, 2, 3, None, None, 6]
    Output: Vec![1, 2, 3, 3, 3, 6]
    */
    let mut last_val: Option<f32> = None;
    for v in v.iter_mut() {
        if v.is_some() {
            last_val = *v;
        }
        if v.is_none() {
            *v = last_val
        }
    }
}
 */
fn create_idx_col_mapping(prop_indicies: &Vec<i32>) -> (HashMap<i32, usize>, HashMap<usize, i32>) {
    /*
    Create mapping from property index into column index.
    This is needed because prop indicies might be:
    24, 248, 354 and we can't be creating 354 columns so
    we map it into 0,1,2..
    */
    println!("PROP IDXX{:?}", prop_indicies);
    let mut idx_pos: HashMap<i32, usize> = HashMap::default();
    let mut col_idx: HashMap<usize, i32> = HashMap::default();

    for (cnt, p_idx) in prop_indicies.iter().enumerate() {
        idx_pos.insert(*p_idx, cnt);
        col_idx.insert(cnt, *p_idx);
    }
    println!("MAP TO IDXX{:?}", idx_pos);
    (idx_pos, col_idx)
}
fn col_str_mapping(col_indicies: &Vec<i32>, parser_maps: Arc<RwLock<ParsingMaps>>) -> Vec<String> {
    /*
    Maps column index to it's human readable name
    */
    let parser_maps_read = parser_maps.read().unwrap();
    let serverclass_map = parser_maps_read.serverclass_map.as_ref().unwrap();

    let mut str_names = vec![];
    let props = &serverclass_map.player.props;
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

pub fn str_props_to_int_props(
    str_props: &Vec<String>,
    parser_maps: Arc<RwLock<ParsingMaps>>,
) -> Vec<i32> {
    let parser_maps_read = parser_maps.read().unwrap();
    let serverclass_map = parser_maps_read.serverclass_map.as_ref().unwrap();

    let mut int_props = vec![];
    for s in str_props {
        if s != "m_vecOrigin" {
            int_props.push(str_prop_to_int(s, &serverclass_map));
        }
    }

    println!("STR {:?}", str_props);
    println!("INT {:?}", int_props);
    int_props
}

fn str_prop_to_int(wanted_prop: &str, serverclass_map: &ServerClasses) -> i32 {
    /*
    Maps string names to their prop index (used in packet ents)
    For example m_angEyeAngles[0] --> 20
    Mainly need to pay attention to manager props comming from different
    serverclass
    */

    match wanted_prop {
        "m_vecOrigin_X" => return 10000i32,
        "m_vecOrigin_Y" => return 10001i32,
        "m_vecVelocity[0]" => return 2i32,
        "m_vecVelocity[1]" => return 3i32,
        "m_vecVelocity[2]" => return 4i32,
        _ => {}
    }

    let sv_cls = &serverclass_map.player;

    match sv_cls.props.iter().position(|x| x.name == wanted_prop) {
        Some(idx) => (idx + 5) as i32,
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
