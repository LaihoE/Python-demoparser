use crate::parsing::entities::Entity;
use hashbrown::HashMap;

pub fn extract_props(
    entities: &Option<HashMap<u32, Option<Entity>>>,
    props_names: &Vec<String>,
    tick: &i32,
) -> Vec<(String, f32)> {
    let mut tick_props: Vec<(String, f32)> = Vec::new();

    if entities.is_some() {
        if entities.as_ref().unwrap().contains_key(&7) {
            if entities.as_ref().unwrap()[&7].is_some() {
                let x = entities.as_ref().unwrap()[&7].as_ref().unwrap();

                for prop_name in props_names {
                    if x.props.contains_key(prop_name) {
                        tick_props.push((prop_name.to_string(), x.props[prop_name].data.to_float()))
                    } else {
                        tick_props.push((prop_name.to_string(), -1.0))
                    }
                }
                tick_props.push(("tick".to_string(), *tick as f32))
            }
        }
    }
    tick_props
}
