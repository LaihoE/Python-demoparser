use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::Entity;
use crate::parsing::parser::Parser;
use crate::parsing::stringtables::UserInfo;
use crate::parsing::variants;
use crate::parsing::variants::PropColumn;
use crate::parsing::variants::PropData::I32;
use crate::parsing::variants::VarVec;
use ahash::RandomState;
use phf::phf_map;
use std::collections::HashMap;
use std::collections::HashSet;

use super::variants::PropData;

#[inline(always)]
pub fn create_default(col_type: i32, playback_frames: usize) -> PropColumn {
    let v = match col_type {
        0 => VarVec::I32(Vec::with_capacity(playback_frames)),
        1 => VarVec::F32(Vec::with_capacity(playback_frames)),
        2 => VarVec::F32(Vec::with_capacity(playback_frames)),
        4 => VarVec::String(Vec::with_capacity(playback_frames)),
        5 => VarVec::U64(Vec::with_capacity(playback_frames)),
        10 => VarVec::I32(Vec::with_capacity(playback_frames)),
        _ => panic!("INCORRECT COL TYPE"),
    };
    PropColumn { data: v }
}

#[inline(always)]
fn insert_propcolumn(
    ticks_props: &mut HashMap<String, PropColumn, RandomState>,
    ent: &Entity,
    prop_name: &String,
    playback_frames: usize,
    col_type: i32,
) {
    match ent.props.get(prop_name) {
        None => ticks_props
            .entry(prop_name.to_string())
            .or_insert_with(|| create_default(col_type, playback_frames))
            .data
            .push_none(),
        Some(p) => ticks_props
            .entry(prop_name.to_string())
            .or_insert_with(|| create_default(col_type, playback_frames))
            .data
            .push_propdata(p.data.clone()),
    }
}
#[inline(always)]
fn insert_weapon_prop(
    ticks_props: &mut HashMap<String, PropColumn, RandomState>,
    _ent: &Entity,
    prop_name: &String,
    playback_frames: usize,
    _col_type: i32,
    weapon: Option<&Entity>,
) {
    match weapon {
        Some(w) => match w.props.get(prop_name) {
            Some(w) => {
                {
                    let data = if let PropData::I32(x) = w.data {
                        PropData::I32(x - 1)
                    } else {
                        // Should not happen
                        w.data.clone()
                    };
                    ticks_props
                        .entry(prop_name.to_string())
                        .or_insert_with(|| create_default(0, playback_frames))
                        .data
                        .push_propdata(data)
                }
            }
            None => ticks_props
                .entry(prop_name.to_string())
                .or_insert_with(|| create_default(0, playback_frames))
                .data
                .push_none(),
        },
        None => ticks_props
            .entry(prop_name.to_string())
            .or_insert_with(|| create_default(0, playback_frames))
            .data
            .push_none(),
    }
}
#[inline(always)]
fn insert_weapon_name(
    cls_map: &HashMap<u16, ServerClass, RandomState>,
    ticks_props: &mut HashMap<String, PropColumn, RandomState>,
    ent: &Entity,
    prop_name: &String,
    playback_frames: usize,
    _col_type: i32,
    weapon: Option<&Entity>,
) {
    match weapon {
        Some(w) => match w.props.get("m_iItemDefinitionIndex") {
            Some(w) => {
                if let PropData::I32(x) = w.data {
                    let name = WEAPINDICIES[&x.to_string()];
                    ticks_props
                        .entry(prop_name.to_string())
                        .or_insert_with(|| create_default(4, playback_frames))
                        .data
                        .push_propdata(PropData::String(name.to_string()))
                }
            }
            None => match weapon {
                Some(we) => match cls_map.get(&(we.class_id as u16)) {
                    Some(sc) => {
                        let full_name = sc.dt.to_string();
                        let weap_name = match full_name.split("Weapon").last() {
                            Some(w) => {
                                if w == "M4A1" {
                                    "M4A4"
                                } else {
                                    match full_name.split("_").last() {
                                        Some(x) => x,
                                        None => &full_name,
                                    }
                                }
                            }
                            None => match full_name.split("_").last() {
                                Some(x) => x,
                                None => &full_name,
                            },
                        };
                        ticks_props
                            .entry(prop_name.to_string())
                            .or_insert_with(|| create_default(4, playback_frames))
                            .data
                            .push_propdata(variants::PropData::String(weap_name.to_string()))
                    }
                    None => ticks_props
                        .entry(prop_name.to_string())
                        .or_insert_with(|| create_default(4, playback_frames))
                        .data
                        .push_none(),
                },
                None => ticks_props
                    .entry(prop_name.to_string())
                    .or_insert_with(|| create_default(4, playback_frames))
                    .data
                    .push_none(),
            },
        },
        None => ticks_props
            .entry(prop_name.to_string())
            .or_insert_with(|| create_default(4, playback_frames))
            .data
            .push_none(),
    }
}
#[inline(always)]
fn insert_manager_prop(
    ticks_props: &mut HashMap<String, PropColumn, RandomState>,
    ent: &Entity,
    prop_name: &String,
    playback_frames: usize,
    col_type: i32,
    manager: Option<&Entity>,
) {
    match manager {
        Some(m) => {
            let key = if ent.entity_id < 10 {
                prop_name.to_owned() + "00" + &ent.entity_id.to_string()
            } else if ent.entity_id < 100 {
                prop_name.to_owned() + "0" + &ent.entity_id.to_string()
            } else {
                panic!("Entity id 100 ????: id:{}", ent.entity_id);
            };
            match m.props.get(&key) {
                Some(p) => ticks_props
                    .entry(prop_name.to_string())
                    .or_insert_with(|| create_default(col_type, playback_frames))
                    .data
                    .push_propdata(p.data.clone()),
                None => ticks_props
                    .entry(prop_name.to_string())
                    .or_insert_with(|| create_default(col_type, playback_frames))
                    .data
                    .push_none(),
            }
        }
        None => ticks_props
            .entry(prop_name.to_string())
            .or_insert_with(|| create_default(col_type, playback_frames))
            .data
            .push_none(),
    }
}
#[inline(always)]
fn weap_id_from_ent(ent: &Entity) -> Option<u32> {
    match ent.props.get("m_hActiveWeapon") {
        None => None,
        Some(w) => match w.data {
            I32(i) => {
                return Some((i & 0x7FF) as u32);
            }
            _ => {
                return None;
            }
        },
    }
}
impl Parser {
    #[inline(always)]
    pub fn collect_player_data(
        players: &HashMap<u64, UserInfo, RandomState>,
        tick: &i32,
        wanted_ticks: &HashSet<i32, RandomState>,
        wanted_players: &Vec<u64>,
        entities: &mut Vec<(u32, Entity)>,
        props_names: &Vec<String>,
        ticks_props: &mut HashMap<String, PropColumn, RandomState>,
        playback_frames: usize,
        manager_id: &Option<u32>,
        cls_map: &HashMap<u16, ServerClass, RandomState>,
    ) {
        // Collect wanted props from players
        for player in players.values() {
            if player.xuid == 0 || player.name == "GOTV" {
                continue;
            };
            // Check that we want the tick
            if wanted_ticks.contains(tick) || wanted_ticks.is_empty() {
                // Check that we want the player
                if wanted_players.contains(&player.xuid) || wanted_players.is_empty() {
                    let pl = &mut entities[player.entity_id as usize];
                    if pl.0 != 1111111 {
                        let ent = &entities[player.entity_id as usize];
                        let manager = if manager_id.is_some() {
                            Some(&entities[manager_id.unwrap() as usize].1)
                        } else {
                            None
                        };
                        let weapon_ent = match weap_id_from_ent(&ent.1) {
                            None => None,
                            Some(ent_id) => match entities[ent_id as usize].0 {
                                1111111 => None,
                                _ => Some(&entities[ent_id as usize].1),
                            },
                        };
                        for prop_name in props_names {
                            match TYPEHM[prop_name] {
                                10 => {
                                    insert_manager_prop(
                                        ticks_props,
                                        &ent.1,
                                        prop_name,
                                        playback_frames,
                                        0,
                                        manager,
                                    );
                                }
                                20 => insert_weapon_prop(
                                    ticks_props,
                                    &ent.1,
                                    prop_name,
                                    playback_frames,
                                    0,
                                    weapon_ent,
                                ),
                                99 => insert_weapon_name(
                                    cls_map,
                                    ticks_props,
                                    &ent.1,
                                    prop_name,
                                    playback_frames,
                                    0,
                                    weapon_ent,
                                ),
                                _ => {
                                    insert_propcolumn(
                                        ticks_props,
                                        &ent.1,
                                        prop_name,
                                        playback_frames,
                                        TYPEHM[prop_name],
                                    );
                                }
                            }
                        }
                        // Insert tick, steamid, name
                        insert_metadata(
                            player.name.clone(),
                            *tick,
                            player.xuid,
                            ticks_props,
                            playback_frames,
                        )
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn insert_metadata(
    name: String,
    tick: i32,
    xuid: u64,
    ticks_props: &mut HashMap<String, PropColumn, RandomState>,
    playback_frames: usize,
) {
    ticks_props
        .entry("tick".to_string())
        .or_insert_with(|| create_default(0, playback_frames))
        .data
        .push_i32(tick);

    ticks_props
        .entry("name".to_string())
        .or_insert_with(|| create_default(4, playback_frames))
        .data
        .push_string(name.to_string());

    ticks_props
        .entry("steamid".to_string())
        .or_insert_with(|| create_default(5, playback_frames))
        .data
        .push_u64(xuid);
}

// Found in scripts/items/items_game.txt
pub static WEAPINDICIES: phf::Map<&'static str, &'static str> = phf_map! {
    "default" => "default",
    "1" => "deagle",
    "2" => "elite",
    "3" => "fiveseven",
    "4" => "glock",
    "7" => "ak47",
    "8" => "aug",
    "9" => "awp",
    "10" => "famas",
    "11" => "g3sg1",
    "13" => "galilar",
    "14" => "m249",
    "16" => "m4a1",
    "17" => "mac10",
    "19" => "p90",
    "20" => "zone_repulsor",
    "23" => "mp5sd",
    "24" => "ump45",
    "25" => "xm1014",
    "26" => "bizon",
    "27" => "mag7",
    "28" => "negev",
    "29" => "sawedoff",
    "30" => "tec9",
    "31" => "taser",
    "32" => "hkp2000",
    "33" => "mp7",
    "34" => "mp9",
    "35" => "nova",
    "36" => "p250",
    "37" => "shield",
    "38" => "scar20",
    "39" => "sg556",
    "40" => "ssg08",
    "41" => "knifegg",
    "42" => "knife",
    "43" => "flashbang",
    "44" => "hegrenade",
    "45" => "smokegrenade",
    "46" => "molotov",
    "47" => "decoy",
    "48" => "incgrenade",
    "49" => "c4",
    "50" => "item_kevlar",
    "51" => "item_assaultsuit",
    "52" => "item_heavyassaultsuit",
    "54" => "item_nvg",
    "55" => "item_defuser",
    "56" => "item_cutters",
    "57" => "healthshot",
    "58" => "musickit_default",
    "59" => "knife_t",
    "60" => "m4a1_silencer",
    "61" => "usp_silencer",
    "62" => "Recipe Trade Up",
    "63" => "cz75a",
    "64" => "revolver",
    "68" => "tagrenade",
    "69" => "fists",
    "70" => "breachcharge",
    "72" => "tablet",
    "74" => "melee",
    "75" => "axe",
    "76" => "hammer",
    "78" => "spanner",
    "80" => "knife_ghost",
    "81" => "firebomb",
    "82" => "diversion",
    "83" => "frag_grenade",
    "84" => "snowball",
    "85" => "bumpmine",
    "500" => "bayonet",
    "503" => "knife_css",
    "505" => "knife_flip",
    "506" => "knife_gut",
    "507" => "knife_karambit",
    "508" => "knife_m9_bayonet",
    "509" => "knife_tactical",
    "512" => "knife_falchion",
    "514" => "knife_survival_bowie",
    "515" => "knife_butterfly",
    "516" => "knife_push",
    "517" => "knife_cord",
    "518" => "knife_canis",
    "519" => "knife_ursus",
    "520" => "knife_gypsy_jackknife",
    "521" => "knife_outdoor",
    "522" => "knife_stiletto",
    "523" => "knife_widowmaker",
    "525" => "knife_skeleton",
};

pub static TYPEHM: phf::Map<&'static str, i32> = phf_map! {
    "m_flNextAttack" => 1,
    "m_bDuckOverride" => 0,
    "m_flStamina" => 1,
    "m_flVelocityModifier" => 1,
    "m_iShotsFired" => 0,
    "m_nQuestProgressReason" => 0,
    "m_vecOrigin" => 2,
    "m_vecOrigin_X" => 1,
    "m_vecOrigin_Y" => 1,
    "m_vecOrigin[2]" => 1,
    "m_aimPunchAngle" => 2,
    "m_aimPunchAngle_X" => 1,
    "m_aimPunchAngle_Y" => 1,
    "m_aimPunchAngleVel" => 2,
    "m_aimPunchAngleVel_X" => 1,
    "m_aimPunchAngleVel_Y" => 1,
    "m_audio.soundscapeIndex" => 0,
    "m_bDucked" => 0,
    "m_bDucking" => 0,
    "m_bWearingSuit" => 0,
    "m_chAreaBits.000" => 0,
    "m_chAreaBits.001" => 0,
    "m_chAreaPortalBits.002" => 0,
    "m_flFOVRate" => 1,
    "m_flFallVelocity" => 1,
    "m_flLastDuckTime" => 1,
    "m_viewPunchAngle" => 2,
    "m_viewPunchAngle_X" => 1,
    "m_viewPunchAngle_Y" => 1,
    "m_flDeathTime" => 1,
    "m_flNextDecalTime" => 1,
    "m_hLastWeapon" => 0,
    "m_hTonemapController" => 0,
    "m_nNextThinkTick" => 0,
    "m_nTickBase" => 0,
    "m_vecBaseVelocity" => 2,
    "m_vecBaseVelocity_X" => 1,
    "m_vecBaseVelocity_Y" => 1,
    "m_vecVelocity[0]" => 1,
    "m_vecVelocity[1]" => 1,
    "m_vecVelocity[2]" => 1,
    "m_vecViewOffset[2]" => 1,
    "m_ArmorValue" => 0,
    "m_usSolidFlags" => 0,
    "m_vecMaxs" => 2,
    "m_vecMaxs_X" => 1,
    "m_vecMaxs_Y" => 1,
    "m_vecMins" => 2,
    "m_vecMins_X" => 1,
    "m_vecMins_Y" => 1,
    "m_LastHitGroup" => 0,
    "m_afPhysicsFlags" => 0,
    "m_angEyeAngles[0]" => 1,
    "m_angEyeAngles[1]" => 1,
    "m_bAnimatedEveryTick" => 0,
    "m_bClientSideRagdoll" => 0,
    "m_bHasDefuser" => 0,
    "m_bHasHelmet" => 0,
    "m_bHasMovedSinceSpawn" => 0,
    "m_bInBombZone" => 0,
    "m_bInBuyZone" => 0,
    "m_bIsDefusing" => 0,
    "m_bIsHoldingLookAtWeapon" => 0,
    "m_bIsLookingAtWeapon" => 0,
    "m_bIsScoped" => 0,
    "m_bIsWalking" => 0,
    "m_bResumeZoom" => 0,
    "m_bSpotted" => 0,
    "m_bSpottedByMask.000" => 0,
    "m_bStrafing" => 0,
    "m_bWaitForNoAttack" => 0,
    "m_fEffects" => 0,
    "m_fFlags" => 0,
    "m_fMolotovDamageTime" => 1,
    "m_fMolotovUseTime" => 1,
    "m_flDuckAmount" => 1,
    "m_flDuckSpeed" => 1,
    "m_flFOVTime" => 1,
    "m_flFlashDuration" => 1,
    "m_flFlashMaxAlpha" => 1,
    "m_flGroundAccelLinearFracLastTime" => 1,
    "m_flLastMadeNoiseTime" => 1,
    "m_flLowerBodyYawTarget" => 1,
    "m_flProgressBarStartTime" => 1,
    "m_flSimulationTime" => 0,
    "m_flThirdpersonRecoil" => 1,
    "m_flTimeOfLastInjury" => 1,
    "m_hActiveWeapon" => -1,
    "m_hColorCorrectionCtrl" => 0,
    "m_hGroundEntity" => 0,
    "m_hMyWeapons.000" => 0,
    "m_hMyWeapons.001" => 0,
    "m_hMyWeapons.002" => 0,
    "m_hMyWeapons.003" => 0,
    "m_hMyWeapons.004" => 0,
    "m_hMyWeapons.005" => 0,
    "m_hMyWeapons.006" => 0,
    "m_hMyWeapons.007" => 0,
    "m_hMyWeapons.008" => 0,
    "m_hObserverTarget" => 0,
    "m_hPlayerPing" => 0,
    "m_hPostProcessCtrl" => 0,
    "m_hRagdoll" => 0,
    "m_hViewModel" => 5,
    "m_hZoomOwner" => 0,
    "m_iAccount" => 0,
    "m_iAddonBits" => 0,
    "m_iAmmo.014" => 0,
    "m_iAmmo.015" => 0,
    "m_iAmmo.016" => 0,
    "m_iAmmo.017" => 0,
    "m_iAmmo.018" => 0,
    "m_iClass" => 0,
    "m_iDeathPostEffect" => 0,
    "m_iFOV" => 0,
    "m_iFOVStart" => 0,
    "m_iHealth" => 0,
    "m_iMoveState" => 0,
    "m_iNumRoundKills" => 0,
    "m_iNumRoundKillsHeadshots" => 0,
    "m_iObserverMode" => 0,
    "m_iPendingTeamNum" => 0,
    "m_iPlayerState" => 0,
    "m_iPrimaryAddon" => 0,
    "m_iProgressBarDuration" => 0,
    "m_iSecondaryAddon" => 0,
    "m_iStartAccount" => 0,
    "m_iTeamNum" => 0,
    "m_lifeState" => 0,
    "m_nForceBone" => 0,
    "m_nHeavyAssaultSuitCooldownRemaining" => 0,
    "m_nLastConcurrentKilled" => 0,
    "m_nLastKillerIndex" => 0,
    "m_nModelIndex" => 0,
    "m_nRelativeDirectionOfLastInjury" => 0,
    "m_nWaterLevel" => 0,
    "m_rank.005" => 0,
    "m_szLastPlaceName" => 4,
    "m_totalHitsOnServer" => 0,
    "m_ubEFNoInterpParity" => 0,
    "m_unCurrentEquipmentValue" => 0,
    "m_unFreezetimeEndEquipmentValue" => 0,
    "m_unMusicID" => 0,
    "m_unRoundStartEquipmentValue" => 0,
    "m_unTotalRoundDamageDealt" => 0,
    "m_vecForce" => 2,
    "m_vecForce_X" => 1,
    "m_vecForce_Y" => 1,
    "m_vecLadderNormal" => 2,
    "m_vecLadderNormal_X" => 1,
    "m_vecLadderNormal_Y" => 1,
    "m_vecPlayerPatchEconIndices.002" => 0,
    "movetype" => 0,
    "pl.deadflag" => 0,
    "m_bSilencerOn" => 0,
    "m_bReloadVisuallyComplete" => 1,
    "m_iCompetitiveRanking" => 10,
    "m_iPing" => 10,
    "m_iTeam" => 10,
    "m_iScore" => 10,
    "m_iDeaths" => 10,
    "m_iKills" => 10,
    "m_iAssists" => 10,
    "m_iMVPs" => 10,
    "m_iArmor" => 10,
    "m_iCompetitiveWins" => 10,
    "m_iMatchStats_UtilityDamage_Total" => 10,
    "m_iMatchStats_Damage_Total" => 10,
    "m_iLifetimeStart" => 10,
    "m_iLifetimeEnd" => 10,
    "m_bConnected" => 10,
    "m_bControllingBot" => 10,
    "m_iControlledPlayer"=> 10,
    "m_iControlledByPlayer"=> 10,
    "m_iTotalCashSpent"=> 10,
    "m_iCashSpentThisRound"=> 10,
    "m_nPersonaDataPublicCommendsLeader"=> 10,
    "m_nPersonaDataPublicCommendsTeacher"=> 10,
    "m_nPersonaDataPublicCommendsFriendly"=> 10,
    "m_bHasCommunicationAbuseMute"=> 10,
    "m_iMatchStats_Kills_Total"=> 10,
    "m_iMatchStats_5k_Total"=> 10,
    "m_iMatchStats_4k_Total"=> 10,
    "m_iMatchStats_3k_Total"=> 10,
    "m_iMatchStats_EquipmentValue_Total"=> 10,
    "m_iMatchStats_KillReward_Total"=> 10,
    "m_iMatchStats_LiveTime_Total"=> 10,
    "m_iMatchStats_Deaths_Total"=> 10,
    "m_iMatchStats_Assists_Total"=> 10,
    "m_iMatchStats_HeadShotKills_Total"=> 10,
    "m_iMatchStats_Objective_Total"=> 10,
    "m_iMatchStats_CashEarned_Total"=> 10,
    "m_iMatchStats_EnemiesFlashed_Total"=> 10,
    "m_bInDuckJump"=> 0,
    "m_nDuckJumpTimeMsecs"=> 0,
    "m_nJumpTimeMsecs"=> 0,
    "m_vecViewOffset[1]"=> 1,
    "m_vecViewOffset[0]"=> 1,
    "m_fOnTarget"=> 0,
    "m_flFriction" => 1,
    "m_flElasticity"=> 1,
    "m_iThrowGrenadeCounter" => 0,
    "m_bIsRescuing"=> 0,
    "m_flDetectedByEnemySensorTime"=> 1,
    "m_bHasHeavyArmor"=> 0,
    "m_nActiveCoinRank"=> 0,
    "m_nPersonaDataPublicLevel"=> 0,
    "m_iClip1" => 20,
    "weapon_name" => 99,
    "m_bAlive" => 10,
};
