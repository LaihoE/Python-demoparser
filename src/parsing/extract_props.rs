use crate::parsing::entities::Entity;
use hashbrown::HashMap;

use super::stringtables::UserInfo;

pub fn extract_props(
    entities: &Option<HashMap<u32, Option<Entity>>>,
    props_names: &Vec<String>,
    tick: &i32,
    wanted_id: u32,
) -> Vec<(String, f32)> {
    let mut tick_props: Vec<(String, f32)> = Vec::new();

    if entities.is_some() {
        if entities.as_ref().unwrap().contains_key(&wanted_id) {
            if entities.as_ref().unwrap()[&wanted_id].is_some() {
                let x = entities.as_ref().unwrap()[&wanted_id].as_ref().unwrap();

    match players.get(&76561198194694750) {
        Some(e) => {
            //println!("FOUND PLAYERR");
            let wanted_ent = e;
            let wanted_ent_id = wanted_ent.entity_id;
            if entities.is_some() {
                if entities.as_ref().unwrap().contains_key(&wanted_ent_id) {
                    if entities.as_ref().unwrap()[&wanted_ent_id].is_some() {
                        let ent = entities.as_ref().unwrap()[&wanted_ent_id].as_ref().unwrap();

                        for prop_name in props_names {
                            println!("{}", prop_name);
                            if ent.props.contains_key(prop_name) {
                                tick_props.push((
                                    prop_name.to_string(),
                                    ent.props[prop_name].data.to_float(),
                                ))
                            } else {
                                tick_props.push((prop_name.to_string(), -1.0))
                            }
                        }
                        tick_props.push(("tick".to_string(), *tick as f32))
                    }
                }
                tick_props.push(("tick".to_string(), *tick as f32));
                tick_props.push(("ent_id".to_string(), wanted_id as f32));
            }
        }
        None => {}
    }
    tick_props
}
