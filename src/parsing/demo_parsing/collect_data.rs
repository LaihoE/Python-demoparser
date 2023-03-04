use crate::parsing::demo_parsing::*;
use crate::parsing::parser::Parser;
use crate::parsing::utils::IS_ARRAY_PROP;
use crate::parsing::variants::PropColumn;
use crate::parsing::variants::PropData;
use crate::parsing::variants::VarVec;
use crate::CACHE_ID_MAP;
use ahash::HashMap;
use itertools::Itertools;
use phf::phf_map;

pub const TICK_ID: i32 = -1;
pub const NAME_ID: i32 = -2;
pub const STEAMID_ID: i32 = -3;
pub const AMMO_ID: i32 = -10;
pub const WEAP_NAME_ID: i32 = -20;

#[inline(always)]
pub fn create_default(col_type: i32, playback_frames: usize) -> PropColumn {
    let v = match col_type {
        0 => VarVec::I32(Vec::with_capacity(playback_frames)),
        1 => VarVec::F32(Vec::with_capacity(playback_frames)),
        2 => VarVec::F32(Vec::with_capacity(playback_frames)),
        4 => VarVec::String(Vec::with_capacity(playback_frames)),
        5 => VarVec::U64(Vec::with_capacity(playback_frames)),
        10 => VarVec::I32(Vec::with_capacity(playback_frames)),
        _ => panic!("INCORRECT COL TYPE: {}", col_type),
    };
    PropColumn { data: v }
}

impl Parser {
    #[inline(always)]
    fn get_weapon_ent(&self, ent: &Entity) -> Option<u32> {
        match ent.props.get(self.state.weapon_handle_id as usize) {
            None => None,
            Some(w) => match w {
                Some(PropData::I32(i)) => {
                    return Some((i & 0x7FF) as u32);
                }
                _ => {
                    return None;
                }
            },
        }
    }
    pub fn collect_weapons(&mut self) {
        for (xuid, player) in &self.maps.players {
            if xuid == &0 {
                continue;
            }
            match &self.state.entities.get(&(player.entity_id as i32)) {
                Some(ent) => {
                    let weapon_ent = self.get_weapon_ent(&ent);
                    match weapon_ent {
                        Some(weap_id) => {
                            let weapon_ent = &self.state.entities.get(&(weap_id as i32));
                            let ammo = self.collect_ammo(weapon_ent);
                            let weapon = self.collect_weapon(weapon_ent);
                            match weapon {
                                Some(weap) => {
                                    self.state
                                        .output
                                        .entry(WEAP_NAME_ID)
                                        .or_insert_with(|| create_default(4, 1024))
                                        .data
                                        .push_string(weap);
                                }
                                None => {
                                    self.state
                                        .output
                                        .entry(WEAP_NAME_ID)
                                        .or_insert_with(|| create_default(4, 1024))
                                        .data
                                        .push_string_none();
                                }
                            }
                            match ammo {
                                Some(ammo) => {
                                    self.state
                                        .output
                                        .entry(AMMO_ID)
                                        .or_insert_with(|| create_default(0, 1024))
                                        .data
                                        .push_i32(ammo);
                                }
                                None => {
                                    self.state
                                        .output
                                        .entry(AMMO_ID)
                                        .or_insert_with(|| create_default(0, 1024))
                                        .data
                                        .push_i32_none();
                                }
                            }
                        }
                        None => {
                            self.state
                                .output
                                .entry(AMMO_ID)
                                .or_insert_with(|| create_default(0, 1024))
                                .data
                                .push_i32_none();
                            self.state
                                .output
                                .entry(WEAP_NAME_ID)
                                .or_insert_with(|| create_default(4, 1024))
                                .data
                                .push_string_none();
                        }
                    }
                }
                None => {
                    self.state
                        .output
                        .entry(AMMO_ID)
                        .or_insert_with(|| create_default(0, 1024))
                        .data
                        .push_i32_none();
                    self.state
                        .output
                        .entry(WEAP_NAME_ID)
                        .or_insert_with(|| create_default(4, 1024))
                        .data
                        .push_string_none();
                }
            }
        }
    }

    pub fn find_array_prop(&mut self, entity_id: i32, prop: &String) -> (u32, i32) {
        let key = if entity_id < 10 {
            "00".to_owned() + &entity_id.to_string()
        } else if entity_id < 100 {
            "0".to_owned() + &entity_id.to_string()
        } else {
            panic!("Entity id > 100 ????: id:{}", entity_id);
        };

        let prop_id = self.maps.name_entid_prop[&(prop.to_owned() + &key)];
    }

    pub fn collect_players(&mut self) {
        for prop in &self.settings.collect_props {
            for (xuid, player) in &self.maps.players {
                if xuid == &0 {
                    continue;
                }
                //let (entid, pidx)

                match &self.state.entities.get(&(player.entity_id as i32)) {
                    Some(ent) => match ent.props.get(prop_id as usize).unwrap() {
                        None => self
                            .state
                            .output
                            .entry(prop_id as i32)
                            .or_insert_with(|| create_default(self.maps.name_ptype_map[prop], 1024))
                            .data
                            .push_none(),
                        Some(p) => {
                            self.state
                                .output
                                .entry(prop_id as i32)
                                .or_insert_with(|| {
                                    create_default(self.maps.name_ptype_map[prop], 1024)
                                })
                                .data
                                .push_propdata(p.clone());
                        }
                    },
                    None => {
                        self.state
                            .output
                            .entry(prop_id as i32)
                            .or_insert_with(|| create_default(self.maps.name_ptype_map[prop], 1024))
                            .data
                            .push_none();
                    }
                }
            }
        }
        for (xuid, player) in &self.maps.players {
            if xuid == &0 {
                continue;
            }
            // Metadata
            self.state
                .output
                .entry(-1)
                .or_insert_with(|| create_default(0, 1024))
                .data
                .push_i32(self.state.tick);

            self.state
                .output
                .entry(-2)
                .or_insert_with(|| create_default(4, 1024))
                .data
                .push_string(player.name.to_string());

            self.state
                .output
                .entry(-3)
                .or_insert_with(|| create_default(5, 1024))
                .data
                .push_u64(player.xuid);
        }
    }
    pub fn collect_ammo(&self, weapon_ent: &Option<&Entity>) -> Option<i32> {
        match weapon_ent {
            Some(w) => match w.props.get(self.state.clip_id as usize) {
                Some(w) => {
                    if let Some(PropData::I32(x)) = w {
                        return Some(x - 1);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        None
    }
    pub fn collect_weapon(&self, weapon_ent: &Option<&Entity>) -> Option<String> {
        match weapon_ent {
            Some(weapon) => match &weapon.props[self.state.item_def_id as usize] {
                Some(itemdef) => match itemdef {
                    crate::parsing::variants::PropData::I32(i) => {
                        let weapon_name = match WEAPINDICIES.get(&i.to_string()) {
                            Some(name) => {
                                if name == &"m4a1" {
                                    "m4a4"
                                } else {
                                    name
                                }
                            }
                            None => "MISSING_WEAPON",
                        };
                        return Some(weapon_name.to_string());
                    }

                    _ => match self.maps.serverclass_map.get(&(weapon.class_id as u16)) {
                        None => return None,
                        Some(cls) => {
                            let full_name = cls.dt.to_string();
                            let weapon_name = match full_name.split("Weapon").last() {
                                Some(w) => {
                                    if w == "m4a1" {
                                        "m4a4"
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
                            return Some(weapon_name.to_string());
                        }
                    },
                },
                None => match self.maps.serverclass_map.get(&(weapon.class_id as u16)) {
                    None => return None,
                    Some(cls) => {
                        let full_name = cls.dt.to_string();
                        let weapon_name = match full_name.split("Weapon").last() {
                            Some(w) => {
                                if w == "m4a1" {
                                    "m4a4"
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
                        return Some(weapon_name.to_string());
                    }
                },
            },
            None => None,
        }
    }
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
