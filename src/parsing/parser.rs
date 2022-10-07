use super::game_events::GameEvent;
use super::read_bits::PropData;
use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::Entity;
use crate::parsing::stringtables::StringTable;
use crate::parsing::stringtables::UserInfo;
use csgoproto::netmessages::csvcmsg_game_event_list::Descriptor_t;
use csgoproto::netmessages::*;
use fxhash::FxHashMap;
use hashbrown::HashMap;
use hashbrown::HashSet;
use memmap::Mmap;
use phf::phf_map;
use protobuf;
use protobuf::Message;
use std::io::Read;
use std::ops::Index;
use std::u8;

#[allow(dead_code)]
pub struct Frame {
    pub cmd: u8,
    pub tick: i32,
    pub playerslot: u8,
}
#[derive(Debug, Clone)]
pub enum VarVec {
    U64(Vec<Option<u64>>),
    F32(Vec<Option<f32>>),
    I64(Vec<Option<i64>>),
    I32(Vec<Option<i32>>),
    String(Vec<Option<String>>),
}
#[derive(Debug, Clone)]
pub struct PropColumn {
    pub dtype: String,
    pub data: VarVec,
}

pub struct Demo {
    pub fp: usize,
    pub tick: i32,
    pub cmd: u8,
    pub bytes: BytesVariant,
    pub class_bits: u32,
    pub event_list: Option<CSVCMsg_GameEventList>,
    pub event_map: Option<HashMap<i32, Descriptor_t>>,
    pub dt_map: Option<HashMap<String, CSVCMsg_SendTable>>,
    pub serverclass_map: HashMap<u16, ServerClass>,
    pub entities: Option<HashMap<u32, Option<Entity>>>,
    pub bad: Vec<String>,
    pub stringtables: Vec<StringTable>,
    pub players: HashMap<u64, UserInfo>,
    pub parse_props: bool,
    pub game_events: Vec<GameEvent>,
    pub event_name: String,
    pub cnt: i32,
    pub wanted_props: Vec<String>,
    pub wanted_ticks: HashSet<i32>,
    pub wanted_players: Vec<u64>,
    pub round: i32,
    pub players_connected: i32,
    pub only_players: bool,
    pub only_header: bool,
    pub wanted_ent_ids: Vec<u32>,
    pub playback_frames: usize,
}

impl VarVec {
    pub fn push_propdata(&mut self, item: PropData) {
        match item {
            PropData::F32(p) => match self {
                VarVec::F32(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            PropData::I32(p) => match self {
                VarVec::I32(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            PropData::I64(p) => match self {
                VarVec::I64(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a {:?} column", item, self);
                }
            },
            PropData::String(p) => match self {
                VarVec::String(f) => f.push(Some(p)),
                _ => {
                    panic!("Tried to push a {:?} into a string column", p);
                }
            },
            _ => panic!("bad type for prop"),
        }
    }
    pub fn push_string(&mut self, data: String) {
        match self {
            VarVec::String(f) => f.push(Some(data)),
            _ => {}
        }
    }
    pub fn push_string_none(&mut self) {
        match self {
            VarVec::String(f) => f.push(None),
            _ => {}
        }
    }
    pub fn push_float_none(&mut self) {
        match self {
            VarVec::F32(f) => f.push(None),
            _ => {}
        }
    }
    pub fn push_i32_none(&mut self) {
        match self {
            VarVec::I32(f) => f.push(None),
            _ => {}
        }
    }
}

pub enum BytesVariant {
    Mmap(Mmap),
    Vec(Vec<u8>),
}

impl<Idx> std::ops::Index<Idx> for BytesVariant
where
    Idx: std::slice::SliceIndex<[u8]>,
{
    type Output = Idx::Output;
    #[inline(always)]
    fn index(&self, i: Idx) -> &Self::Output {
        match self {
            Self::Mmap(m) => {
                return &m[i];
            }
            Self::Vec(v) => {
                return &v[i];
            }
        }
    }
}
impl BytesVariant {
    pub fn get_len(&self) -> usize {
        match self {
            Self::Mmap(m) => m.len(),
            Self::Vec(v) => v.len(),
        }
    }
}

impl Demo {
    pub fn new_mmap(
        bytes: Mmap,
        parse_props: bool,
        wanted_ticks: Vec<i32>,
        wanted_players: Vec<u64>,
        mut wanted_props: Vec<String>,
        event_name: String,
        only_players: bool,
        only_header: bool,
    ) -> Result<Self, std::io::Error> {
        let mut extra_wanted_props = vec![];
        for p in &wanted_props {
            match TYPEHM.get(&p) {
                Some(n) => {
                    if &p[(p.len() - 1)..] == "X" {
                        extra_wanted_props.push((&p[..p.len() - 2]).to_owned());
                    } else if &p[(p.len() - 1)..] == "Y" {
                        extra_wanted_props.push((&p[..p.len() - 2]).to_owned());
                    } else if &p[(p.len() - 1)..] == "Z" {
                        extra_wanted_props.push((&p[..p.len() - 2]).to_owned());
                    }
                }
                None => {
                    panic!("Prop: {} not found", p);
                }
            }
        }
        wanted_props.extend(extra_wanted_props);
        Ok(Self {
            wanted_ent_ids: Vec::new(),
            bytes: BytesVariant::Mmap(bytes),
            fp: 0,
            cmd: 0,
            tick: 0,
            cnt: 0,
            round: 0,
            event_list: None,
            event_map: None,
            class_bits: 0,
            parse_props: parse_props,
            event_name: event_name,
            bad: Vec::new(),
            dt_map: Some(HashMap::default()),
            serverclass_map: HashMap::default(),
            entities: Some(HashMap::default()),
            stringtables: Vec::new(),
            players: HashMap::new(),
            wanted_props: wanted_props,
            game_events: Vec::new(),
            wanted_players: wanted_players,
            wanted_ticks: HashSet::from_iter(wanted_ticks),
            players_connected: 0,
            only_header: only_header,
            only_players: only_players,
            playback_frames: 0,
        })
    }
    pub fn new(
        bytes: Vec<u8>,
        parse_props: bool,
        wanted_ticks: Vec<i32>,
        wanted_players: Vec<u64>,
        mut wanted_props: Vec<String>,
        event_name: String,
        only_players: bool,
        only_header: bool,
    ) -> Result<Self, std::io::Error> {
        let mut extra_wanted_props = vec![];
        for p in &wanted_props {
            match TYPEHM.get(&p) {
                Some(n) => {
                    if &p[(p.len() - 1)..] == "X" {
                        extra_wanted_props.push((&p[..p.len() - 2]).to_owned());
                    } else if &p[(p.len() - 1)..] == "Y" {
                        extra_wanted_props.push((&p[..p.len() - 2]).to_owned());
                    } else if &p[(p.len() - 1)..] == "Z" {
                        extra_wanted_props.push((&p[..p.len() - 2]).to_owned());
                    }
                }
                None => {
                    panic!("Prop: {} not found", p);
                }
            }
        }
        wanted_props.extend(extra_wanted_props);
        Ok(Self {
            wanted_ent_ids: Vec::new(),
            bytes: BytesVariant::Vec(bytes),
            fp: 0,
            cmd: 0,
            tick: 0,
            cnt: 0,
            round: 0,
            event_list: None,
            event_map: None,
            class_bits: 0,
            parse_props: parse_props,
            event_name: event_name,
            bad: Vec::new(),
            dt_map: Some(HashMap::default()),
            serverclass_map: HashMap::default(),
            entities: Some(HashMap::default()),
            stringtables: Vec::new(),
            players: HashMap::new(),
            wanted_props: wanted_props,
            game_events: Vec::new(),
            wanted_players: wanted_players,
            wanted_ticks: HashSet::from_iter(wanted_ticks),
            players_connected: 0,
            only_header: only_header,
            only_players: only_players,
            playback_frames: 0,
        })
    }
}

impl Demo {
    pub fn parse_frame(&mut self, props_names: &Vec<String>) -> FxHashMap<String, PropColumn> {
        let mut ticks_props: FxHashMap<String, PropColumn> = FxHashMap::default();
        while self.fp < self.bytes.get_len() as usize {
            let f = self.read_frame_bytes();
            self.tick = f.tick;
            // EARLY EXIT
            if self.only_players {
                if Demo::all_players_connected(self.players_connected) {
                    break;
                }
            }
            // EARLY EXIT
            if self.only_header {
                break;
            }
            for (_, player) in &self.players {
                if player.xuid == 0 || player.name == "GOTV" {
                    continue;
                };
                if self.wanted_ticks.contains(&self.tick) || self.wanted_ticks.len() == 0 {
                    if self.wanted_players.contains(&player.xuid) || self.wanted_players.len() == 0
                    {
                        if self
                            .entities
                            .as_ref()
                            .unwrap()
                            .contains_key(&player.entity_id)
                        {
                            //println!("X");
                            if self.entities.as_ref().unwrap()[&player.entity_id].is_some() {
                                let ent = self.entities.as_ref().unwrap()[&player.entity_id]
                                    .as_ref()
                                    .unwrap();

                                for prop_name in props_names {
                                    let prop_type = TYPEHM[prop_name];
                                    match ent.props.get(prop_name) {
                                        None => {
                                            match prop_type {
                                                // INT
                                                0 => {
                                                    ticks_props
                                                        .entry(prop_name.to_string())
                                                        .or_insert_with(|| PropColumn {
                                                            dtype: "f32".to_string(),
                                                            data: VarVec::I32(Vec::with_capacity(
                                                                self.playback_frames,
                                                            )),
                                                        })
                                                        .data
                                                        .push_i32_none();
                                                }
                                                // FLOAT
                                                1 => {
                                                    ticks_props
                                                        .entry(prop_name.to_string())
                                                        .or_insert_with(|| PropColumn {
                                                            dtype: "f32".to_string(),
                                                            data: VarVec::F32(Vec::with_capacity(
                                                                self.playback_frames,
                                                            )),
                                                        })
                                                        .data
                                                        .push_float_none();
                                                }
                                                // Vec
                                                2 => {
                                                    ticks_props
                                                        .entry(prop_name.to_string())
                                                        .or_insert_with(|| PropColumn {
                                                            dtype: "f32".to_string(),
                                                            data: VarVec::F32(Vec::with_capacity(
                                                                self.playback_frames,
                                                            )),
                                                        })
                                                        .data
                                                        .push_float_none();
                                                }
                                                // STRING
                                                4 => {
                                                    ticks_props
                                                        .entry(prop_name.to_string())
                                                        .or_insert_with(|| PropColumn {
                                                            dtype: "f32".to_string(),
                                                            data: VarVec::String(
                                                                Vec::with_capacity(
                                                                    self.playback_frames,
                                                                ),
                                                            ),
                                                        })
                                                        .data
                                                        .push_string_none();
                                                }
                                                _ => {
                                                    println!("UNK TYPE");
                                                }
                                            }
                                        }
                                        Some(e) => match prop_type {
                                            0 => {
                                                ticks_props
                                                    .entry(prop_name.to_string())
                                                    .or_insert_with(|| PropColumn {
                                                        dtype: "f32".to_string(),
                                                        data: VarVec::I32(Vec::with_capacity(
                                                            self.playback_frames,
                                                        )),
                                                    })
                                                    .data
                                                    .push_propdata(e.data.clone());
                                            }
                                            1 => {
                                                ticks_props
                                                    .entry(prop_name.to_string())
                                                    .or_insert_with(|| PropColumn {
                                                        dtype: "f32".to_string(),
                                                        data: VarVec::F32(Vec::with_capacity(
                                                            self.playback_frames,
                                                        )),
                                                    })
                                                    .data
                                                    .push_propdata(e.data.clone());
                                            }
                                            4 => {
                                                ticks_props
                                                    .entry(prop_name.to_string())
                                                    .or_insert_with(|| PropColumn {
                                                        dtype: "f32".to_string(),
                                                        data: VarVec::String(Vec::with_capacity(
                                                            self.playback_frames,
                                                        )),
                                                    })
                                                    .data
                                                    .push_propdata(e.data.clone());
                                            }
                                            _ => panic!("UNKOWN PROP TYPE"),
                                        },
                                    }
                                }
                                // EXTRA
                                ticks_props
                                    .entry("tick".to_string())
                                    .or_insert_with(|| PropColumn {
                                        dtype: "i32".to_string(),
                                        data: VarVec::String(Vec::with_capacity(
                                            self.playback_frames,
                                        )),
                                    })
                                    .data
                                    .push_string(self.tick.to_string());

                                ticks_props
                                    .entry("steamid".to_string())
                                    .or_insert_with(|| PropColumn {
                                        dtype: "u64".to_string(),
                                        data: VarVec::String(Vec::with_capacity(
                                            self.playback_frames,
                                        )),
                                    })
                                    .data
                                    .push_string(player.xuid.to_string());
                                ticks_props
                                    .entry("name".to_string())
                                    .or_insert_with(|| PropColumn {
                                        dtype: "u64".to_string(),
                                        data: VarVec::String(Vec::with_capacity(
                                            self.playback_frames,
                                        )),
                                    })
                                    .data
                                    .push_string(player.name.to_string());
                            }
                        }
                    }
                }
            }
            self.parse_cmd(f.cmd);
        }
        ticks_props
    }

    pub fn parse_cmd(&mut self, cmd: u8) {
        match cmd {
            1 => self.parse_packet(),
            2 => self.parse_packet(),
            6 => self.parse_datatable(),
            _ => {
                //println!("CMD {}", cmd); //panic!("UNK CMD")
            } //,
        }
    }

    pub fn all_players_connected(total_connected: i32) -> bool {
        if total_connected == 10 {
            return true;
        }
        return false;
    }

    pub fn parse_packet(&mut self) {
        self.fp += 160;
        let packet_len = self.read_i32();
        let goal_inx = self.fp + packet_len as usize;
        let parse_props = self.parse_props;
        while self.fp < goal_inx {
            let msg = self.read_varint();
            let size = self.read_varint();
            let data = self.read_n_bytes(size);

            match msg as i32 {
                // Game event
                25 => {
                    let game_event = Message::parse_from_bytes(&data);
                    match game_event {
                        Ok(ge) => {
                            let game_event = ge;
                            let game_events = self.parse_game_events(game_event);
                            self.game_events.extend(game_events);
                        }
                        Err(e) => panic!(
                            "Failed to parse game event at tick {}. Error: {e}",
                            self.tick
                        ),
                    }
                }
                // Game event list
                30 => {
                    let event_list = Message::parse_from_bytes(&data);
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
                // Packet entites
                26 => {
                    if parse_props {
                        let pack_ents = Message::parse_from_bytes(&data);
                        match pack_ents {
                            Ok(pe) => {
                                let pack_ents = pe;
                                self.parse_packet_entities(pack_ents, parse_props);
                            }
                            Err(e) => panic!(
                                "Failed to parse Packet entities at tick {}. Error: {e}",
                                self.tick
                            ),
                        }
                    }
                }
                // Create string table
                12 => {
                    let string_table = Message::parse_from_bytes(&data);
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
                    let data = Message::parse_from_bytes(&data);
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
};
