use crate::parsing::demo_parsing::*;
use crate::parsing::parser::Parser;
use crate::parsing::variants::PropColumn;
use crate::parsing::variants::VarVec;
use ahash::HashMap;

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

impl Parser {
    pub fn collect_data(&mut self) {
        let wanted_id = 1229;
        for (xuid, player) in &self.maps.players {
            match &self.state.entities.get(&(player.entity_id as i32)) {
                Some(ent) => match ent.props.get(wanted_id as usize).unwrap() {
                    None => self
                        .state
                        .output
                        .entry(wanted_id)
                        .or_insert_with(|| create_default(0, 1024))
                        .data
                        .push_none(),
                    Some(p) => self
                        .state
                        .output
                        .entry(wanted_id)
                        .or_insert_with(|| create_default(0, 1024))
                        .data
                        .push_propdata(p.clone()),
                },
                None => {}
            }
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
}
