use crate::parsing::entities::Entity;
pub use crate::parsing::variants::*;
use flate2::read::GzDecoder;
use memmap2::MmapOptions;
use phf::phf_map;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::u8;

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
    let extension = Path::new(&demo_path).extension().unwrap();
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
    "m_iClip1" => 0,
    "weapon_name" => 99,
    "m_bAlive" => 10,

};
