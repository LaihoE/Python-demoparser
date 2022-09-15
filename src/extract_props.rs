use crate::Entity;
use std::collections::HashMap;

pub fn extract_props(
    entities: &Option<HashMap<u32, Option<Entity>>>,
    props_names: &Vec<String>,
) -> HashMap<String, Vec<f32>> {
    let mut tick_props: Vec<(String, Vec<f32>)> = Vec::new();

    if entities.is_some() {
        if entities.as_ref().unwrap().contains_key(&6) {
            if entities.as_ref().unwrap()[&6].is_some() {
                let x = entities.as_ref().unwrap()[&6].as_ref().unwrap();

                for prop_name in props_names {
                    if x.props.contains_key(prop_name) {
                        tick_props.push((prop_name.to_string(), x.props[prop_name].data.to_float()))
                    } else {
                        tick_props
                            .entry(prop_name.to_string())
                            .or_insert(Vec::new())
                            .push(-1.0);
                    }
                }
            }
        }
    }
    //println!("{:?}", tick_props);
    tick_props
}
