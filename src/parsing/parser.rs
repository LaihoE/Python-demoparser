use super::game_events::GameEvent;
use super::tick_cache;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::Entity;
use crate::parsing::game_events::KeyData;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
use crate::parsing::tick_cache::TickCache;
pub use crate::parsing::variants::*;
use ahash::RandomState;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use flate2::read::GzDecoder;
use memmap::MmapOptions;
use mimalloc::MiMalloc;
use phf::phf_map;
use protobuf;
use protobuf::Message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::u8;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub struct Parser {
    pub maps: Maps,
    pub settings: ParserSettings,
    pub fp: usize,
    pub tick: i32,
    pub bytes: BytesVariant,
    pub round: i32,
    pub entities: Vec<(u32, Entity)>,
    pub stringtables: Vec<StringTable>,
    pub game_events: Vec<GameEvent>,
    // Optimization
    pub workhorse: Vec<i32>,
    pub friendly_p_names: Vec<String>,
}

pub struct Maps {
    /*
    Different lookup maps used during parsing
    */
    pub serverclass_map: HashMap<u16, ServerClass, RandomState>,
    pub event_map: Option<HashMap<i32, Descriptor_t, RandomState>>,
    pub dt_map: Option<HashMap<String, CSVCMsg_SendTable, RandomState>>,
    pub players: HashMap<u64, UserInfo, RandomState>,
    pub userid_sid_map: HashMap<u32, Vec<(u64, i32)>, RandomState>,
    pub uid_eid_map: HashMap<u32, Vec<(u32, i32)>, RandomState>,
    pub baselines: HashMap<u32, HashMap<String, PropData>>,
    pub baseline_no_cls: HashMap<u32, Vec<u8>>,
    pub sid_entid_map: HashMap<u64, Vec<(u32, i32)>>,
}

pub struct ParserSettings {
    pub only_players: bool,
    pub only_header: bool,
    pub parse_props: bool,
    pub event_name: String,
    pub no_gameevents: bool,
    pub early_exit_tick: i32,
    pub wanted_props: Vec<String>,
    pub wanted_ticks: HashSet<i32, RandomState>,
    pub wanted_players: Vec<u64>,
    pub playback_frames: usize,
}

impl Parser {
    pub fn new(
        demo_path: String,
        parse_props: bool,
        wanted_ticks: Vec<i32>,
        wanted_players: Vec<u64>,
        mut wanted_props: Vec<String>,
        event_name: String,
        only_players: bool,
        only_header: bool,
        no_gameevents: bool,
        early_exit_tick: i32,
        og_names: Vec<String>,
    ) -> Result<Self, std::io::Error> {
        let mut extra_wanted_props = vec![];
        for p in &wanted_props {
            match TYPEHM.get(p) {
                Some(_) => match &p[(p.len() - 1)..] {
                    "X" => extra_wanted_props.push((&p[..p.len() - 2]).to_owned()),
                    "Y" => extra_wanted_props.push((&p[..p.len() - 2]).to_owned()),
                    "Z" => extra_wanted_props.push((&p[..p.len() - 2]).to_owned()),
                    _ => {}
                },
                None => {
                    panic!("Prop: {} not found", p);
                }
            }
        }

        wanted_props.extend(extra_wanted_props);
        match read_file(demo_path) {
            Err(e) => Err(e),
            Ok(data) => {
                let maps = Maps {
                    serverclass_map: HashMap::default(),
                    event_map: Some(HashMap::default()),
                    dt_map: Some(HashMap::default()),
                    players: HashMap::default(),
                    userid_sid_map: HashMap::default(),
                    baselines: HashMap::default(),
                    baseline_no_cls: HashMap::default(),
                    sid_entid_map: HashMap::default(),
                    uid_eid_map: HashMap::default(),
                };
                let settings = ParserSettings {
                    only_players: only_players,
                    only_header: only_header,
                    parse_props: parse_props,
                    event_name: event_name,
                    no_gameevents: no_gameevents,
                    early_exit_tick: early_exit_tick,
                    wanted_props: wanted_props,
                    wanted_ticks: HashSet::from_iter(wanted_ticks.iter().cloned()),
                    wanted_players: wanted_players,
                    playback_frames: 0,
                };

                Ok(Self {
                    maps: maps,
                    settings: settings,
                    bytes: data,
                    fp: 0,
                    tick: 0,
                    round: 0,
                    entities: vec![],
                    stringtables: Vec::new(),
                    game_events: Vec::new(),
                    workhorse: Vec::new(),
                    friendly_p_names: og_names,
                })
            }
        }
    }
}

impl Parser {
    pub fn start_parsing(
        &mut self,
        props_names: &Vec<String>,
    ) -> (HashMap<String, PropColumn, RandomState>, TickCache) {
        let mut ticks_props: HashMap<String, PropColumn, RandomState> = HashMap::default();
        let mut tc = TickCache::new();
        for _ in 0..10000 {
            self.entities.push((
                1111111,
                Entity {
                    class_id: 0,
                    entity_id: 1111111,
                    props: HashMap::default(),
                },
            ));
        }
        for i in 0..20000 {
            self.workhorse.push(i);
        }

        while self.fp < self.bytes.get_len() as usize {
            let (cmd, tick) = self.read_frame();
            if tick > self.settings.early_exit_tick {
                break;
            }
            self.tick = tick;
            // EARLY EXIT
            if self.settings.only_header {
                break;
            }
            /*
            if self.parse_props {
                Demo::collect_player_data(
                    &self.players,
                    &self.tick,
                    &self.wanted_ticks,
                    &self.wanted_players,
                    &mut self.entities,
                    props_names,
                    &mut ticks_props,
                    self.playback_frames,
                    &self.manager_id,
                    &self.serverclass_map,
                );
            }
            */
            self.parse_cmd(cmd, &mut tc);
        }
        (ticks_props, tc)
    }
    #[inline(always)]
    pub fn parse_cmd(&mut self, cmd: u8, tc: &mut TickCache) {
        match cmd {
            1 => self.parse_packet(tc),
            2 => self.parse_packet(tc),
            6 => self.parse_datatable(),
            _ => {}
        }
    }

    #[inline(always)]
    pub fn parse_packet(&mut self, tc: &mut TickCache) {
        check_round_change(&self.entities, &mut self.round);
        self.fp += 160;
        let packet_len = self.read_i32();
        let goal_inx = self.fp + packet_len as usize;
        let parse_props = self.settings.parse_props;
        let mut is_con_tick = false;
        let no_gameevents = self.settings.no_gameevents;
        let t = self.tick;
        while self.fp < goal_inx {
            let msg = self.read_varint();
            let size = self.read_varint();
            let before_inx = self.fp.clone();
            let data = self.read_n_bytes(size);
            if msg == 26 {
                tc.insert_tick(t, before_inx, before_inx + size as usize);
            }
            match msg as i32 {
                // Game event
                25 => {
                    if !no_gameevents {
                        let game_event = Message::parse_from_bytes(data);
                        match game_event {
                            Ok(ge) => {
                                let game_event = ge;
                                let (game_events, con_tick) = self.parse_game_events(game_event);
                                is_con_tick = con_tick;
                                self.game_events.extend(game_events);
                            }
                            Err(e) => panic!(
                                "Failed to parse game event at tick {}. Error: {e}",
                                self.tick
                            ),
                        }
                    }
                }
                // Game event list
                30 => {
                    if !no_gameevents {
                        let event_list = Message::parse_from_bytes(data);
                        match event_list {
                            Ok(ev) => {
                                let event_list = ev;
                                self.parse_game_event_map(event_list)
                            }
                            Err(e) => panic!(
                                "Failed to parse game event LIST at tick {}. Error: {e}",
                                self.tick
                            ),
                        }
                    }
                }
                // Packet entites
                26 => {
                    if t < -5555555 {
                        if parse_props {
                            let pack_ents = Message::parse_from_bytes(data);
                            match pack_ents {
                                Ok(pe) => {
                                    let pack_ents = pe;
                                    Parser::parse_packet_entities(
                                        &mut self.maps.serverclass_map,
                                        self.tick,
                                        pack_ents,
                                        &mut self.entities,
                                        &self.settings.wanted_props,
                                        &mut self.workhorse,
                                        self.fp as i32,
                                        &mut self.round,
                                        &self.maps.baselines,
                                    );
                                }
                                Err(e) => panic!(
                                    "Failed to parse Packet entities at tick {}. Error: {e}",
                                    self.tick
                                ),
                            }
                        }
                    }
                }
                // Create string table
                12 => {
                    let string_table = Message::parse_from_bytes(data);
                    match string_table {
                        Ok(st) => {
                            let string_table = st;
                            self.create_string_table(string_table);
                        }
                        Err(e) => panic!(
                            "Failed to parse String table at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                // Update string table
                13 => {
                    let data = Message::parse_from_bytes(data);
                    match data {
                        Ok(st) => {
                            let data = st;
                            self.update_string_table_msg(data);
                        }
                        Err(e) => panic!(
                            "Failed to parse String table at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                _ => {}
            }
        }
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
            Ok(m) => Ok(BytesVariant::Mmap(m)),
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

};
