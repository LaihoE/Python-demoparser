use crate::parsing::demo_parsing::*;
use crate::parsing::parser::Parser;
pub use crate::parsing::variants::*;
use ahash::HashMap;
use flate2::read::GzDecoder;
use memmap2::MmapOptions;
use phf::phf_map;
use pyo3::Py;
use pyo3::PyAny;
use pyo3::ToPyObject;
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
        let mut hm: HashMap<String, String> = HashMap::default();
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
    pub fn generate_name_id_map(&mut self) -> HashMap<String, usize> {
        let mut mapping = HashMap::default();

        for (k, m) in &self.maps.serverclass_map {
            for (idx, p) in m.props.iter().enumerate() {
                if k != &40 && !(p.name == "m_iClip1" || p.name == "m_iItemDefinitionIndex") {
                    continue;
                }
                if p.name == "m_hActiveWeapon" && k == &40 {
                    self.state.weapon_handle_id = idx as i32;
                }
                mapping.insert(p.name.clone(), idx);
            }
        }
        mapping.insert("X".to_string(), 4999);
        mapping.insert("Y".to_string(), 4998);
        mapping
    }
    pub fn generate_name_ptype_map(&mut self) -> HashMap<String, i32> {
        let mut mapping = HashMap::default();

        for (k, m) in &self.maps.serverclass_map {
            for (idx, p) in m.props.iter().enumerate() {
                if k != &40 && !(p.name == "m_iClip1" || p.name == "m_iItemDefinitionIndex") {
                    continue;
                }
                mapping.insert(p.name.clone(), p.p_type);
            }
        }
        mapping.insert("X".to_string(), 1);
        mapping.insert("Y".to_string(), 1);
        mapping
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
pub static CACHE_ID_MAP: phf::Map<&'static str, i32> = phf_map! {
"pl.deadflag" => 0,
"m_iControlledByPlayer" => 1,
"m_bIsHoldingLookAtWeapon" => 2,
"m_flThirdpersonRecoil" => 3,
"m_flStamina" => 4,
"m_vecPlayerPatchEconIndices.002" => 5,
"m_flElasticity" => 6,
"m_bHasHelmet" => 7,
"m_hActiveWeapon" => 8,
"m_bAnimatedEveryTick" => 9,
"m_vecOrigin_X" => 10,
"m_nRelativeDirectionOfLastInjury" => 11,
"m_vecVelocity[0]" => 12,
"m_iMatchStats_Kills_Total" => 13,
"m_iAssists" => 14,
"m_iPlayerState" => 15,
"m_nModelIndex" => 16,
"m_bResumeZoom" => 17,
"m_vecViewOffset[0]" => 18,
"m_iFOVStart" => 19,
"m_totalHitsOnServer" => 20,
"m_nJumpTimeMsecs" => 21,
"m_bHasMovedSinceSpawn" => 22,
"m_bClientSideRagdoll" => 23,
"m_hObserverTarget" => 24,
"m_bWearingSuit" => 25,
"m_vecVelocity[2]" => 26,
"m_vecBaseVelocity_X" => 27,
"m_aimPunchAngle_Y" => 28,
"m_hPostProcessCtrl" => 29,
"movetype" => 30,
"m_flGroundAccelLinearFracLastTime" => 31,
"m_vecMaxs_Y" => 32,
"m_iDeathPostEffect" => 33,
"m_bSilencerOn" => 34,
"m_nHeavyAssaultSuitCooldownRemaining" => 35,
"m_rank.005" => 36,
"m_iPrimaryAddon" => 37,
"m_fMolotovDamageTime" => 38,
"m_aimPunchAngleVel" => 39,
"m_flFlashDuration" => 40,
"m_nPersonaDataPublicCommendsTeacher" => 41,
"m_hZoomOwner" => 42,
"m_unMusicID" => 43,
"m_hViewModel" => 44,
"m_bSpotted" => 45,
"m_iMatchStats_UtilityDamage_Total" => 46,
"m_aimPunchAngleVel_X" => 47,
"m_nPersonaDataPublicLevel" => 48,
"m_iAddonBits" => 49,
"m_iAmmo.017" => 50,
"m_nLastKillerIndex" => 51,
"m_flSimulationTime" => 52,
"m_iMVPs" => 53,
"m_iMoveState" => 54,
"m_vecMaxs" => 55,
"m_flFOVTime" => 56,
"m_vecVelocity[1]" => 57,
"m_iMatchStats_Damage_Total" => 58,
"m_afPhysicsFlags" => 59,
"m_iNumRoundKillsHeadshots" => 60,
"m_bStrafing" => 61,
"m_iProgressBarDuration" => 62,
"m_iKills" => 63,
"m_lifeState" => 64,
"m_flLowerBodyYawTarget" => 65,
"m_ArmorValue" => 66,
"m_unRoundStartEquipmentValue" => 67,
"m_flDuckSpeed" => 68,
"m_iPing" => 69,
"m_bSpottedByMask.000" => 70,
"m_chAreaBits.001" => 71,
"m_hLastWeapon" => 72,
"m_unTotalRoundDamageDealt" => 73,
"m_vecBaseVelocity_Y" => 74,
"m_iLifetimeEnd" => 75,
"m_bReloadVisuallyComplete" => 76,
"m_hRagdoll" => 77,
"m_nQuestProgressReason" => 78,
"m_vecOrigin" => 79,
"m_vecMins_X" => 80,
"m_iScore" => 81,
"m_flDeathTime" => 82,
"m_chAreaBits.000" => 83,
"m_hPlayerPing" => 84,
"m_iAmmo.016" => 85,
"m_bDucking" => 86,
"m_fMolotovUseTime" => 87,
"m_hTonemapController" => 88,
"m_nDuckJumpTimeMsecs" => 89,
"m_vecForce" => 90,
"m_flVelocityModifier" => 91,
"m_iAmmo.015" => 92,
"m_iTeam" => 93,
"m_flDetectedByEnemySensorTime" => 94,
"m_bIsLookingAtWeapon" => 95,
"m_iCompetitiveRanking" => 96,
"m_vecLadderNormal_Y" => 97,
"m_viewPunchAngle" => 98,
"m_iClip1" => 99,
"m_vecViewOffset[1]" => 100,
"m_vecBaseVelocity" => 101,
"m_iShotsFired" => 102,
"m_iMatchStats_5k_Total" => 103,
"m_bHasHeavyArmor" => 104,
"m_bIsWalking" => 105,
"m_bDuckOverride" => 106,
"m_iClass" => 107,
"m_viewPunchAngle_Y" => 108,
"m_nTickBase" => 109,
"m_bHasDefuser" => 110,
"m_iMatchStats_3k_Total" => 111,
"m_nWaterLevel" => 112,
"m_angEyeAngles[0]" => 113,
"m_nActiveCoinRank" => 114,
"m_flProgressBarStartTime" => 115,
"m_fOnTarget" => 116,
"m_iHealth" => 117,
"m_bIsScoped" => 118,
"m_iFOV" => 119,
"m_nNextThinkTick" => 120,
"m_vecOrigin_Y" => 121,
"m_fFlags" => 122,
"m_iMatchStats_EnemiesFlashed_Total" => 123,
"m_vecMins" => 124,
"m_bWaitForNoAttack" => 125,
"m_nPersonaDataPublicCommendsFriendly" => 126,
"m_unFreezetimeEndEquipmentValue" => 127,
"m_iMatchStats_CashEarned_Total" => 128,
"m_chAreaPortalBits.002" => 129,
"m_flFallVelocity" => 130,
"m_iMatchStats_LiveTime_Total" => 131,
"m_ubEFNoInterpParity" => 132,
"m_flTimeOfLastInjury" => 133,
"m_iCompetitiveWins" => 134,
"weapon_name" => 135,
"m_bIsRescuing" => 136,
"m_iMatchStats_KillReward_Total" => 137,
"m_iMatchStats_EquipmentValue_Total" => 138,
"m_flLastDuckTime" => 139,
"m_iPendingTeamNum" => 140,
"m_iAccount" => 141,
"m_flFriction" => 142,
"m_iMatchStats_4k_Total" => 143,
"m_iTotalCashSpent" => 144,
"m_bHasCommunicationAbuseMute" => 145,
"m_vecForce_X" => 146,
"m_bInDuckJump" => 147,
"m_szLastPlaceName" => 148,
"m_bConnected" => 149,
"m_flDuckAmount" => 150,
"m_hGroundEntity" => 151,
"m_iMatchStats_HeadShotKills_Total" => 152,
"m_bDucked" => 153,
"m_bInBuyZone" => 154,
"m_vecOrigin[2]" => 155,
"m_flFlashMaxAlpha" => 156,
"m_vecViewOffset[2]" => 157,
"m_angEyeAngles[1]" => 158,
"m_iArmor" => 159,
"m_iMatchStats_Assists_Total" => 160,
"m_vecLadderNormal_X" => 161,
"m_iLifetimeStart" => 162,
"m_nPersonaDataPublicCommendsLeader" => 163,
"m_vecForce_Y" => 164,
"m_vecMins_Y" => 165,
"m_iControlledPlayer" => 166,
"m_LastHitGroup" => 167,
"m_flLastMadeNoiseTime" => 168,
"m_iObserverMode" => 169,
"m_iTeamNum" => 170,
"m_hColorCorrectionCtrl" => 171,
"round" => 172,
"m_aimPunchAngle_X" => 173,
"m_nLastConcurrentKilled" => 174,
"m_nForceBone" => 175,
"m_viewPunchAngle_X" => 176,
"m_aimPunchAngleVel_Y" => 177,
"m_iStartAccount" => 178,
"m_usSolidFlags" => 179,
"m_iThrowGrenadeCounter" => 180,
"m_flNextDecalTime" => 181,
"m_iMatchStats_Objective_Total" => 182,
"m_iNumRoundKills" => 183,
"m_bIsDefusing" => 184,
"m_vecMaxs_X" => 185,
"m_iCashSpentThisRound" => 186,
"m_iDeaths" => 187,
"m_aimPunchAngle" => 188,
"m_vecLadderNormal" => 189,
"m_iSecondaryAddon" => 190,
"m_iMatchStats_Deaths_Total" => 191,
"m_iAmmo.018" => 192,
"m_fEffects" => 193,
"m_flFOVRate" => 194,
"m_iAmmo.014" => 195,
"m_flNextAttack" => 196,
"m_bInBombZone" => 197,
"m_bControllingBot" => 198,
"m_unCurrentEquipmentValue" => 199,
"ammo" => 200,
"weapon" => 201,
};
