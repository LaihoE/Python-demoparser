use crate::parsing::demo_parsing::*;
use crate::parsing::parser::Parser;
pub use crate::parsing::variants::*;
use flate2::read::GzDecoder;
use memmap2::MmapOptions;
use phf::phf_map;
use pyo3::Py;
use pyo3::PyAny;
use pyo3::ToPyObject;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str;
use std::u8;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Header {
    pub header_magic: String,
    pub protocol: i32,
    pub network_protocol: u32,
    pub server_name: String,
    pub client_name: String,
    pub map_name: String,
    pub game_dir: String,
    pub playback_time: f32,
    pub playback_ticks: i32,
    pub playback_frames: i32,
    pub signon_length: i32,
}

impl Header {
    fn to_hashmap(&self) -> HashMap<String, String> {
        let mut hm: HashMap<String, String> = HashMap::new();
        hm.insert("protocol".to_string(), self.protocol.to_string());
        hm.insert(
            "network_protocol".to_string(),
            self.network_protocol.to_string(),
        );
        hm.insert("server_name".to_string(), self.server_name.to_string());
        hm.insert("client_name".to_string(), self.client_name.to_string());
        hm.insert("map_name".to_string(), self.map_name.to_string());
        hm.insert("game_dir".to_string(), self.game_dir.to_string());
        hm.insert("playback_time".to_string(), self.playback_time.to_string());
        hm.insert(
            "protoplayback_tickscol".to_string(),
            self.playback_ticks.to_string(),
        );
        hm.insert(
            "playback_frames".to_string(),
            self.playback_frames.to_string(),
        );
        hm.insert("signon_length".to_string(), self.signon_length.to_string());
        hm
    }
    pub fn to_py_hashmap(&self) -> Py<PyAny> {
        let hm = self.to_hashmap();
        pyo3::Python::with_gil(|py| hm.to_object(py))
    }
}

impl Parser {
    pub fn parse_demo_header(&mut self) -> Header {
        let h = Header {
            header_magic: str::from_utf8(&self.bytes[..8])
                .unwrap()
                .trim_end_matches("\x00")
                .to_string(),
            protocol: i32::from_le_bytes(self.bytes[8..12].try_into().unwrap()),
            network_protocol: u32::from_le_bytes(self.bytes[12..16].try_into().unwrap()),
            server_name: str::from_utf8(&self.bytes[16..276])
                .unwrap()
                .to_string()
                .trim_end_matches("\x00")
                .to_string(),
            client_name: str::from_utf8(&self.bytes[276..536])
                .unwrap()
                .to_string()
                .trim_end_matches("\x00")
                .to_string(),
            map_name: str::from_utf8(&self.bytes[536..796])
                .unwrap()
                .to_string()
                .trim_end_matches("\x00")
                .to_string(),
            game_dir: str::from_utf8(&self.bytes[796..1056])
                .unwrap()
                .to_string()
                .trim_end_matches("\x00")
                .to_string(),
            playback_time: f32::from_le_bytes(self.bytes[1056..1060].try_into().unwrap()),
            playback_ticks: i32::from_le_bytes(self.bytes[1060..1064].try_into().unwrap()),
            playback_frames: i32::from_le_bytes(self.bytes[1064..1068].try_into().unwrap()),
            signon_length: i32::from_le_bytes(self.bytes[1068..1072].try_into().unwrap()),
        };
        self.state.fp += 1072_usize;
        h
    }
}

pub fn check_round_change(entities: &[(u32, Entity)], round: &mut i32) {
    match entities.get(70) {
        Some(e) => match e.1.props.get("m_totalRoundsPlayed") {
            Some(r) => {
                if let PropData::I32(p) = r.data {
                    *round = p;
                }
            }
            None => {}
        },
        None => {}
    }
}
pub fn decompress_gz(demo_path: String) -> Result<BytesVariant, std::io::Error> {
    match File::open(demo_path.clone()) {
        Err(e) => Err(e),
        Ok(_) => match std::fs::read(demo_path) {
            Err(e) => Err(e),
            Ok(bytes) => {
                let mut gz = GzDecoder::new(&bytes[..]);
                let mut out: Vec<u8> = vec![];
                gz.read_to_end(&mut out).unwrap();
                Ok(BytesVariant::Vec(out))
            }
        },
    }
}
pub fn create_mmap(demo_path: String) -> Result<BytesVariant, std::io::Error> {
    match File::open(demo_path) {
        Err(e) => Err(e),
        Ok(f) => match unsafe { MmapOptions::new().map(&f) } {
            Err(e) => Err(e),
            Ok(m) => {
                //m.advise(memmap2::Advice::Random).unwrap();
                Ok(BytesVariant::Mmap3(m))
            }
        },
    }
}

pub fn read_file(demo_path: String) -> Result<BytesVariant, std::io::Error> {
    let extension = match Path::new(&demo_path).extension() {
        Some(ext) => ext,
        None => panic!("Could not read file: {}", demo_path),
    };
    match extension.to_str().unwrap() {
        "gz" => match decompress_gz(demo_path) {
            Err(e) => Err(e),
            Ok(bytes) => Ok(bytes),
        },
        ".info" => {
            panic!("you passed an .info file, these are not demos")
        }
        // All other formats, .dem is the "correct" but let others work too
        _ => match create_mmap(demo_path) {
            Err(e) => Err(e),
            Ok(map) => Ok(map),
        },
    }
}
pub static TYPEHM: phf::Map<&'static str, i32> = phf_map! {
"player@m_vecOrigin_X" => 1,
"player@m_vecOrigin_Y" => 1,
"player@DT_Animationlayer.m_flWeight" => 1,
"player@m_iWeaponPurchasesThisMatch.060" => 0,
"player@DT_CSPlayer.m_fMolotovDamageTime" => 1,
"player@m_iMatchStats_Deaths.008" => 0,
"player@m_iMatchStats_HeadShotKills.009" => 0,
"player@m_iMatchStats_Kills.010" => 0,
"player@m_iMatchStats_LiveTime.005" => 0,
"player@m_iMatchStats_EquipmentValue.012" => 0,
"player@m_EquippedLoadoutItemDefIndices.024" => 0,
"player@m_iMatchStats_Assists.002" => 0,
"player@m_iMatchStats_EnemiesFlashed.009" => 0,
"player@m_iMatchStats_Kills.012" => 0,
"player@DT_Animationlayer.m_flPlaybackRate" => 1,
"player@DT_BaseEntity.m_bAnimatedEveryTick" => 0,
"player@DT_LocalPlayerExclusive.m_vecBaseVelocity" => 2,
"player@m_iWeaponPurchasesThisMatch.033" => 0,
"player@DT_CSNonLocalPlayerExclusive.m_vecOrigin" => 3,
"player@m_iWeaponPurchasesThisRound.045" => 0,
"player@m_iMatchStats_LiveTime.002" => 0,
"player@m_iWeaponPurchasesThisRound.036" => 0,
"player@m_iWeaponPurchasesThisMatch.009" => 0,
"player@m_iMatchStats_LiveTime.007" => 0,
"player@DT_CSPlayer.m_hPlayerPing" => 0,
"player@m_EquippedLoadoutItemDefIndices.005" => 0,
"player@m_EquippedLoadoutItemDefIndices.035" => 0,
"player@DT_Local.m_flFallVelocity" => 1,
"player@DT_CSPlayer.m_iMoveState" => 0,
"player@DT_CSPlayer.m_bHasMovedSinceSpawn" => 0,
"player@m_EquippedLoadoutItemDefIndices.008" => 0,
"player@m_bSpottedByMask.000" => 0,
"player@m_iWeaponPurchasesThisMatch.043" => 0,
"player@m_iMatchStats_Damage.006" => 0,
"player@m_iMatchStats_UtilityDamage.012" => 0,
"player@m_iWeaponPurchasesThisRound.060" => 0,
"player@DT_CSNonLocalPlayerExclusive.m_vecOrigin[2]" => 1,
"player@m_iMatchStats_Kills.007" => 0,
"player@m_hMyWeapons.003" => 0,
"player@m_iMatchStats_EnemiesFlashed.001" => 0,
"player@m_iWeaponPurchasesThisMatch.040" => 0,
"player@m_iMatchStats_Kills.009" => 0,
"player@DT_LocalPlayerExclusive.m_vecViewOffset[2]" => 1,
"player@DT_CSPlayer.m_angEyeAngles[0]" => 1,
"player@m_iMatchStats_HeadShotKills.001" => 0,
"player@m_EquippedLoadoutItemDefIndices.004" => 0,
"player@m_iMatchStats_EquipmentValue.006" => 0,
"player@DT_CollisionProperty.m_usSolidFlags" => 0,
"player@DT_CSPlayer.m_iStartAccount" => 0,
"player@DT_CSLocalPlayerExclusive.m_bDuckOverride" => 0,
"player@m_iMatchStats_Kills.004" => 0,
"player@DT_BaseEntity.m_iTeamNum" => 0,
"player@DT_CSPlayer.m_flFlashMaxAlpha" => 1,
"player@m_iMatchStats_Damage.001" => 0,
"player@DT_BasePlayer.m_hObserverTarget" => 0,
"player@DT_CSPlayer.m_bInBombZone" => 0,
"player@m_hMyWeapons.004" => 0,
"player@DT_BasePlayer.m_lifeState" => 0,
"player@DT_CSPlayer.m_nLastKillerIndex" => 0,
"player@m_iMatchStats_Deaths.009" => 0,
"player@DT_CSPlayer.m_unFreezetimeEndEquipmentValue" => 0,
"player@m_iWeaponPurchasesThisMatch.034" => 0,
"player@m_iWeaponPurchasesThisRound.016" => 0,
"player@m_EquippedLoadoutItemDefIndices.001" => 0,
"player@DT_CSPlayer.m_bIsWalking" => 0,
"player@m_iMatchStats_KillReward.003" => 0,
"player@m_EquippedLoadoutItemDefIndices.009" => 0,
"player@m_iMatchStats_Deaths.003" => 0,
"player@DT_LocalPlayerExclusive.m_flDeathTime" => 1,
"player@m_iMatchStats_Deaths.011" => 0,
"player@m_EquippedLoadoutItemDefIndices.020" => 0,
"player@DT_Local.m_skybox3d.fog.enable" => 0,
"player@DT_CSPlayer.m_bIsScoped" => 0,
"player@DT_PlayerState.deadflag" => 0,
"player@DT_Animationlayer.m_nSequence" => 0,
"player@m_iMatchStats_MoneySaved.006" => 0,
"player@DT_Animationlayer.m_nOrder" => 0,
"player@DT_CSPlayer.m_iNumRoundKillsHeadshots" => 0,
"player@DT_Animationlayer.m_flWeightDeltaRate" => 1,
"player@m_iWeaponPurchasesThisMatch.048" => 0,
"player@DT_LocalPlayerExclusive.m_hTonemapController" => 0,
"player@DT_BaseCombatCharacter.m_flTimeOfLastInjury" => 1,
"player@DT_LocalPlayerExclusive.m_vecVelocity[1]" => 1,
"player@DT_Local.m_viewPunchAngle" => 2,
"player@m_iMatchStats_HeadShotKills.003" => 0,
"player@m_iWeaponPurchasesThisMatch.044" => 0,
"player@DT_CSPlayer.m_flGroundAccelLinearFracLastTime" => 1,
"player@m_iMatchStats_Objective.003" => 0,
"player@DT_BaseAnimating.m_vecForce" => 2,
"player@m_iMatchStats_EquipmentValue.004" => 0,
"player@m_iMatchStats_KillReward.004" => 0,
"player@m_iMatchStats_HeadShotKills.011" => 0,
"player@m_iWeaponPurchasesThisRound.017" => 0,
"player@DT_Local.m_skybox3d.fog.end" => 1,
"player@m_iWeaponPurchasesThisMatch.007" => 0,
"player@m_iWeaponPurchasesThisRound.030" => 0,
"player@m_iMatchStats_Deaths.004" => 0,
"player@m_iMatchStats_Deaths.002" => 0,
"player@m_iMatchStats_Damage.009" => 0,
"player@DT_CSPlayer.m_flProgressBarStartTime" => 1,
"player@m_iMatchStats_MoneySaved.010" => 0,
"player@m_iMatchStats_EnemiesFlashed.008" => 0,
"player@m_iMatchStats_CashEarned.011" => 0,
"player@m_iMatchStats_MoneySaved.003" => 0,
"player@m_hMyWeapons.000" => 0,
"player@m_iMatchStats_UtilityDamage.008" => 0,
"player@DT_CSPlayer.m_iSecondaryAddon" => 0,
"player@m_iMatchStats_MoneySaved.007" => 0,
"player@m_iWeaponPurchasesThisRound.008" => 0,
"player@m_EquippedLoadoutItemDefIndices.019" => 0,
"player@m_iAmmo.018" => 0,
"player@m_iWeaponPurchasesThisRound.044" => 0,
"player@m_iMatchStats_EquipmentValue.005" => 0,
"player@m_iMatchStats_UtilityDamage.006" => 0,
"player@m_EquippedLoadoutItemDefIndices.029" => 0,
"player@DT_BaseEntity.m_bSpotted" => 0,
"player@m_EquippedLoadoutItemDefIndices.026" => 0,
"player@DT_Animationlayer.m_flCycle" => 1,
"player@m_iMatchStats_Kills.005" => 0,
"player@m_iMatchStats_Kills.008" => 0,
"player@DT_CSPlayer.m_nHeavyAssaultSuitCooldownRemaining" => 0,
"player@DT_Local.m_skybox3d.fog.maxdensity" => 1,
"player@m_iWeaponPurchasesThisRound.048" => 0,
"player@m_iMatchStats_Objective.004" => 0,
"player@DT_CSPlayer.m_iProgressBarDuration" => 0,
"player@m_iWeaponPurchasesThisMatch.046" => 0,
"player@m_EquippedLoadoutItemDefIndices.027" => 0,
"player@m_hMyWeapons.006" => 0,
"player@m_iMatchStats_Assists.003" => 0,
"player@DT_CSPlayer.m_unRoundStartEquipmentValue" => 0,
"player@m_iMatchStats_Kills.003" => 0,
"player@DT_LocalPlayerExclusive.m_vecVelocity[0]" => 1,
"player@DT_Local.m_skybox3d.scale" => 0,
"player@DT_BaseEntity.movetype" => 0,
"player@DT_BaseEntity.m_flLastMadeNoiseTime" => 1,
"player@DT_BasePlayer.m_ubEFNoInterpParity" => 0,
"player@DT_BasePlayer.m_iFOVStart" => 0,
"player@m_iMatchStats_Assists.001" => 0,
"player@DT_Local.m_flLastDuckTime" => 1,
"player@m_iMatchStats_Damage.003" => 0,
"player@DT_LocalPlayerExclusive.m_nNextThinkTick" => 0,
"player@m_iWeaponPurchasesThisRound.003" => 0,
"player@m_iWeaponPurchasesThisRound.040" => 0,
"player@m_iMatchStats_CashEarned.003" => 0,
"player@m_iMatchStats_CashEarned.000" => 0,
"player@DT_CSPlayer.m_flLowerBodyYawTarget" => 1,
"player@m_iMatchStats_Kills.001" => 0,
"player@DT_Local.m_aimPunchAngleVel" => 2,
"player@DT_CSPlayer.m_iNumRoundKills" => 0,
"player@m_iMatchStats_MoneySaved.004" => 0,
"player@m_iMatchStats_CashEarned.004" => 0,
"player@m_iMatchStats_HeadShotKills.004" => 0,
"player@DT_LocalPlayerExclusive.m_vecVelocity[2]" => 1,
"player@m_iMatchStats_Assists.004" => 0,
"player@m_iMatchStats_Deaths.006" => 0,
"player@m_iWeaponPurchasesThisMatch.045" => 0,
"player@m_iWeaponPurchasesThisMatch.019" => 0,
"player@DT_Local.m_skybox3d.fog.colorPrimary" => 0,
"player@m_iMatchStats_Kills.006" => 0,
"player@m_iWeaponPurchasesThisRound.047" => 0,
"player@DT_CSPlayer.m_ArmorValue" => 0,
"player@DT_CSPlayer.m_unMusicID" => 0,
"player@DT_CSPlayer.m_fMolotovUseTime" => 1,
"player@m_iMatchStats_Deaths.007" => 0,
"player@m_iMatchStats_HeadShotKills.008" => 0,
"player@m_iMatchStats_MoneySaved.009" => 0,
"player@m_EquippedLoadoutItemDefIndices.012" => 0,
"player@m_iMatchStats_UtilityDamage.007" => 0,
"player@m_chAreaBits.002" => 0,
"player@m_chAreaBits.001" => 0,
"player@m_EquippedLoadoutItemDefIndices.010" => 0,
"player@DT_BaseCombatCharacter.m_hActiveWeapon" => 0,
"player@m_iWeaponPurchasesThisRound.034" => 0,
"player@m_iWeaponPurchasesThisRound.019" => 0,
"player@DT_CSPlayer.m_bIsDefusing" => 0,
"player@DT_CSPlayer.m_bHasHelmet" => 0,
"player@m_iMatchStats_EnemiesFlashed.004" => 0,
"player@m_iWeaponPurchasesThisMatch.023" => 0,
"player@DT_LocalPlayerExclusive.m_flNextDecalTime" => 1,
"player@m_iMatchStats_UtilityDamage.009" => 0,
"player@m_iMatchStats_LiveTime.010" => 0,
"player@m_iMatchStats_HeadShotKills.010" => 0,
"player@m_EquippedLoadoutItemDefIndices.054" => 0,
"player@m_iWeaponPurchasesThisRound.026" => 0,
"player@m_iMatchStats_Objective.001" => 0,
"player@DT_Local.m_aimPunchAngle" => 2,
"player@m_iWeaponPurchasesThisMatch.051" => 0,
"player@DT_LocalPlayerExclusive.m_hLastWeapon" => 0,
"player@DT_BaseEntity.m_fEffects" => 0,
"player@DT_CSPlayer.m_hRagdoll" => 0,
"player@DT_Local.m_flFOVRate" => 1,
"player@DT_CSLocalPlayerExclusive.m_nQuestProgressReason" => 0,
"player@m_EquippedLoadoutItemDefIndices.018" => 0,
"player@m_iMatchStats_EquipmentValue.002" => 0,
"player@DT_CSLocalPlayerExclusive.m_vecOrigin[2]" => 1,
"player@DT_BasePlayer.m_hGroundEntity" => 0,
"player@m_iMatchStats_CashEarned.005" => 0,
"player@m_iMatchStats_KillReward.005" => 0,
"player@m_EquippedLoadoutItemDefIndices.002" => 0,
"player@m_iAmmo.017" => 0,
"player@m_iMatchStats_Deaths.005" => 0,
"player@m_iMatchStats_KillReward.007" => 0,
"player@m_EquippedLoadoutItemDefIndices.022" => 0,
"player@m_iWeaponPurchasesThisMatch.047" => 0,
"player@m_rank.005" => 0,
"player@DT_BaseAnimating.m_nForceBone" => 0,
"player@m_iWeaponPurchasesThisMatch.036" => 0,
"player@m_iWeaponPurchasesThisMatch.030" => 0,
"player@m_iMatchStats_EquipmentValue.011" => 0,
"player@m_EquippedLoadoutItemDefIndices.034" => 0,
"player@m_iMatchStats_Kills.002" => 0,
"player@m_iMatchStats_KillReward.002" => 0,
"player@m_iMatchStats_KillReward.011" => 0,
"player@DT_BasePlayer.m_hViewModel" => 5,
"player@m_iWeaponPurchasesThisMatch.016" => 0,
"player@m_iMatchStats_Assists.009" => 0,
"player@DT_BasePlayer.m_afPhysicsFlags" => 0,
"player@DT_BasePlayer.m_flDuckAmount" => 1,
"player@m_iMatchStats_CashEarned.007" => 0,
"player@DT_CSPlayer.m_angEyeAngles[1]" => 1,
"player@m_iMatchStats_CashEarned.008" => 0,
"player@m_iMatchStats_Damage.011" => 0,
"player@m_iMatchStats_Damage.000" => 0,
"player@m_iMatchStats_Kills.011" => 0,
"player@m_iMatchStats_Assists.008" => 0,
"player@m_iMatchStats_HeadShotKills.012" => 0,
"player@DT_CSPlayer.m_nLastConcurrentKilled" => 0,
"player@m_iMatchStats_UtilityDamage.011" => 0,
"player@DT_Local.m_skybox3d.fog.dirPrimary" => 2,
"player@m_iWeaponPurchasesThisMatch.050" => 0,
"player@m_iMatchStats_EquipmentValue.000" => 0,
"player@m_iWeaponPurchasesThisMatch.008" => 0,
"player@m_hMyWeapons.005" => 0,
"player@DT_Local.m_iHideHUD" => 0,
"player@m_iMatchStats_EquipmentValue.010" => 0,
"player@m_iMatchStats_EquipmentValue.001" => 0,
"player@m_iWeaponPurchasesThisMatch.017" => 0,
"player@m_iMatchStats_HeadShotKills.005" => 0,
"player@m_iMatchStats_LiveTime.009" => 0,
"player@m_iMatchStats_Damage.010" => 0,
"player@m_iMatchStats_LiveTime.011" => 0,
"player@m_iMatchStats_MoneySaved.012" => 0,
"player@DT_CSPlayer.m_bIsLookingAtWeapon" => 0,
"player@DT_BasePlayer.m_hColorCorrectionCtrl" => 0,
"player@m_iMatchStats_MoneySaved.011" => 0,
"player@DT_BasePlayer.m_hPostProcessCtrl" => 0,
"player@DT_CSPlayer.m_bIsHoldingLookAtWeapon" => 0,
"player@DT_Local.m_bDucking" => 0,
"player@DT_CSPlayer.m_totalHitsOnServer" => 0,
"player@m_hMyWeapons.002" => 0,
"player@m_iMatchStats_KillReward.000" => 0,
"player@DT_BasePlayer.m_vecLadderNormal" => 2,
"player@m_iWeaponPurchasesThisRound.050" => 0,
"player@m_iMatchStats_MoneySaved.000" => 0,
"player@m_iMatchStats_EquipmentValue.007" => 0,
"player@m_iMatchStats_EquipmentValue.009" => 0,
"player@m_iMatchStats_Damage.012" => 0,
"player@m_EquippedLoadoutItemDefIndices.021" => 0,
"player@m_iMatchStats_LiveTime.001" => 0,
"player@m_iMatchStats_KillReward.008" => 0,
"player@m_iMatchStats_CashEarned.012" => 0,
"player@DT_CSPlayer.m_unCurrentEquipmentValue" => 0,
"player@m_iMatchStats_HeadShotKills.002" => 0,
"player@DT_CSPlayer.m_unTotalRoundDamageDealt" => 0,
"player@DT_BasePlayer.m_hZoomOwner" => 0,
"player@m_iMatchStats_LiveTime.000" => 0,
"player@DT_Local.m_skybox3d.area" => 0,
"player@m_iWeaponPurchasesThisRound.051" => 0,
"player@DT_CSPlayer.m_iClass" => 0,
"player@m_iMatchStats_Kills.000" => 0,
"player@m_iWeaponPurchasesThisRound.033" => 0,
"player@m_iWeaponPurchasesThisRound.055" => 0,
"player@m_iMatchStats_LiveTime.004" => 0,
"player@m_iAmmo.014" => 0,
"player@m_iWeaponPurchasesThisRound.046" => 0,
"player@m_iMatchStats_UtilityDamage.001" => 0,
"player@m_iMatchStats_MoneySaved.001" => 0,
"player@DT_CSPlayer.m_bStrafing" => 0,
"player@m_EquippedLoadoutItemDefIndices.033" => 0,
"player@m_iMatchStats_LiveTime.012" => 0,
"player@DT_CSPlayer.m_bInBuyZone" => 0,
"player@m_EquippedLoadoutItemDefIndices.023" => 0,
"player@m_EquippedLoadoutItemDefIndices.028" => 0,
"player@DT_CSPlayer.m_bWaitForNoAttack" => 0,
"player@m_EquippedLoadoutItemDefIndices.030" => 0,
"player@m_iMatchStats_Damage.005" => 0,
"player@m_EquippedLoadoutItemDefIndices.025" => 0,
"player@DT_Local.m_bWearingSuit" => 0,
"player@m_iMatchStats_CashEarned.002" => 0,
"player@m_hMyWeapons.001" => 0,
"player@m_iAmmo.015" => 0,
"player@m_iMatchStats_HeadShotKills.006" => 0,
"player@m_iMatchStats_KillReward.006" => 0,
"player@m_EquippedLoadoutItemDefIndices.006" => 0,
"player@m_iMatchStats_CashEarned.013" => 0,
"player@m_iMatchStats_KillReward.001" => 0,
"player@DT_CSPlayer.m_iPrimaryAddon" => 0,
"player@m_iMatchStats_EnemiesFlashed.002" => 0,
"player@m_iMatchStats_UtilityDamage.002" => 0,
"player@m_iWeaponPurchasesThisMatch.003" => 0,
"player@m_EquippedLoadoutItemDefIndices.011" => 0,
"player@m_iMatchStats_Damage.004" => 0,
"player@m_iWeaponPurchasesThisMatch.055" => 0,
"player@m_iMatchStats_KillReward.010" => 0,
"player@m_iMatchStats_Assists.011" => 0,
"player@DT_CSLocalPlayerExclusive.m_iShotsFired" => 0,
"player@m_iMatchStats_EnemiesFlashed.000" => 0,
"player@m_iMatchStats_MoneySaved.002" => 0,
"player@m_iMatchStats_CashEarned.009" => 0,
"player@m_iMatchStats_EnemiesFlashed.007" => 0,
"player@DT_BasePlayer.m_iDeathPostEffect" => 0,
"player@DT_BasePlayer.m_flDuckSpeed" => 1,
"player@m_iMatchStats_Deaths.000" => 0,
"player@m_iMatchStats_Deaths.012" => 0,
"player@m_hMyWeapons.007" => 0,
"player@m_iMatchStats_KillReward.009" => 0,
"player@m_EquippedLoadoutItemDefIndices.003" => 0,
"player@DT_CollisionProperty.m_vecMaxs" => 2,
"player@m_iMatchStats_EnemiesFlashed.005" => 0,
"player@m_iMatchStats_CashEarned.001" => 0,
"player@DT_Local.m_skybox3d.origin" => 2,
"player@m_iWeaponPurchasesThisRound.001" => 0,
"player@DT_Local.m_bDucked" => 0,
"player@m_iMatchStats_MoneySaved.008" => 0,
"player@DT_CSLocalPlayerExclusive.m_flVelocityModifier" => 1,
"player@m_iAmmo.016" => 0,
"player@m_iMatchStats_Damage.002" => 0,
"player@m_iMatchStats_UtilityDamage.003" => 0,
"player@m_EquippedLoadoutItemDefIndices.032" => 0,
"player@m_EquippedLoadoutItemDefIndices.000" => 0,
"player@m_EquippedLoadoutItemDefIndices.017" => 0,
"player@DT_CSPlayer.m_iAccount" => 0,
"player@m_iMatchStats_CashEarned.006" => 0,
"player@DT_CSLocalPlayerExclusive.m_unPlayerTvControlFlags" => 0,
"player@DT_BaseCombatCharacter.m_LastHitGroup" => 0,
"player@m_iWeaponPurchasesThisMatch.026" => 0,
"player@m_iWeaponPurchasesThisRound.007" => 0,
"player@m_iWeaponPurchasesThisMatch.001" => 0,
"player@DT_BaseCombatCharacter.m_nRelativeDirectionOfLastInjury" => 0,
"player@DT_Local.m_audio.entIndex" => 0,
"player@m_iMatchStats_EnemiesFlashed.003" => 0,
"player@m_EquippedLoadoutItemDefIndices.016" => 0,
"player@m_chAreaBits.000" => 0,
"player@DT_BaseEntity.m_iPendingTeamNum" => 0,
"player@DT_BasePlayer.m_iFOV" => 0,
"player@DT_CSPlayer.m_bHasDefuser" => 0,
"player@m_iMatchStats_HeadShotKills.000" => 0,
"player@m_iMatchStats_LiveTime.006" => 0,
"player@DT_BaseAnimating.m_bClientSideRagdoll" => 0,
"player@m_iMatchStats_EnemiesFlashed.006" => 0,
"player@DT_BCCLocalPlayerExclusive.m_flNextAttack" => 1,
"player@m_iMatchStats_Objective.000" => 0,
"player@m_iMatchStats_LiveTime.003" => 0,
"player@DT_Local.m_audio.soundscapeIndex" => 0,
"player@DT_Local.m_skybox3d.fog.start" => 1,
"player@DT_BasePlayer.m_szLastPlaceName" => 4,
"player@m_iMatchStats_Deaths.001" => 0,
"player@m_iMatchStats_EquipmentValue.008" => 0,
"player@m_iMatchStats_EquipmentValue.003" => 0,
"player@m_iMatchStats_MoneySaved.005" => 0,
"player@m_iMatchStats_CashEarned.010" => 0,
"player@m_iMatchStats_KillReward.012" => 0,
"player@m_chAreaPortalBits.000" => 0,
"player@DT_BaseEntity.m_nModelIndex" => 0,
"player@DT_Local.m_skybox3d.fog.colorSecondary" => 0,
"player@m_iWeaponPurchasesThisRound.009" => 0,
"player@m_iWeaponPurchasesThisRound.023" => 0,
"player@m_iMatchStats_Damage.007" => 0,
"player@m_iMatchStats_LiveTime.008" => 0,
"player@DT_BasePlayer.m_fFlags" => 0,
"player@DT_CSPlayer.m_iPlayerState" => 0,
"player@DT_CSPlayer.m_bResumeZoom" => 0,
"player@DT_CSLocalPlayerExclusive.m_flStamina" => 1,
"player@DT_CSPlayer.m_flFlashDuration" => 1,
"player@m_iMatchStats_Assists.006" => 0,
"player@DT_BasePlayer.m_iObserverMode" => 0,
"player@DT_BasePlayer.m_flFOVTime" => 1,
"player@DT_CSPlayer.m_iAddonBits" => 0,
"player@m_iMatchStats_Deaths.010" => 0,
"player@DT_CollisionProperty.m_vecMins" => 2,
"player@m_EquippedLoadoutItemDefIndices.014" => 0,
"player@m_iMatchStats_Damage.008" => 0,
"player@m_EquippedLoadoutItemDefIndices.015" => 0,
"player@m_iWeaponPurchasesThisRound.043" => 0,
"player@DT_BasePlayer.m_iHealth" => 0,
"player@DT_Local.m_skybox3d.fog.HDRColorScale" => 1,
"team@DT_Team" => 0,
"manager@m_iMatchStats_Damage_Total" => 0,
"manager@m_iKills" => 0,
"manager@m_nPersonaDataPublicLevel" => 0,
"manager@m_iPendingTeam" => 0,
"manager@m_iMVPs" => 0,
"manager@m_nEndMatchNextMapVotes" => 0,
"manager@m_iHealth" => 0,
"manager@m_iMatchStats_UtilityDamage_Total" => 0,
"manager@m_bConnected" => 0,
"manager@m_iCompTeammateColor" => 0,
"manager@m_nPersonaDataPublicCommendsLeader" => 0,
"manager@m_iLifetimeStart" => 0,
"manager@m_iMatchStats_EquipmentValue_Total" => 0,
"manager@m_iMatchStats_4k_Total" => 0,
"manager@m_iCompetitiveRankType" => 0,
"manager@m_iTotalCashSpent" => 0,
"manager@m_iTeam" => 0,
"manager@m_iCompetitiveRanking" => 0,
"manager@m_iMatchStats_Kills_Total" => 0,
"manager@m_szCrosshairCodes" => 4,
"manager@m_iPing" => 0,
"manager@m_iMatchStats_KillReward_Total" => 0,
"manager@m_iCompetitiveWins" => 0,
"manager@m_bAlive" => 0,
"manager@m_iMatchStats_CashEarned_Total" => 0,
"manager@m_iMatchStats_LiveTime_Total" => 0,
"manager@m_iScore" => 0,
"manager@m_bHasHelmet" => 0,
"manager@m_iAssists" => 0,
"manager@m_nMusicID" => 0,
"manager@m_nPersonaDataPublicCommendsFriendly" => 0,
"manager@m_iMatchStats_5k_Total" => 0,
"manager@m_bHasDefuser" => 0,
"manager@m_nActiveCoinRank" => 0,
"manager@m_iMatchStats_Objective_Total" => 0,
"manager@m_iCashSpentThisRound" => 0,
"manager@m_nPersonaDataPublicCommendsTeacher" => 0,
"manager@m_nCharacterDefIndex" => 0,
"manager@m_iArmor" => 0,
"manager@m_iMatchStats_Deaths_Total" => 0,
"manager@m_iMatchStats_Assists_Total" => 0,
"manager@m_iMatchStats_3k_Total" => 0,
"manager@m_iLifetimeEnd" => 0,
"manager@m_iMatchStats_HeadShotKills_Total" => 0,
"manager@m_iMatchStats_EnemiesFlashed_Total" => 0,
"manager@m_iDeaths" => 0,
"manager@m_iBotDifficulty" => 0,
"manager@DT_CSPlayerResource" => 0,
"manager@m_szClan" => 4,
"rules@DT_CSGameRules" => 0,
"rules@m_iMatchStats_PlayersAlive_CT" => 0,
"rules@m_iMatchStats_PlayersAlive_T" => 0,
"rules@m_iMatchStats_RoundResults" => 0,
"rules@m_flNextRespawnWave" => 1,
"rules@DT_CSGameRules.m_totalRoundsPlayed" => 0,
};
