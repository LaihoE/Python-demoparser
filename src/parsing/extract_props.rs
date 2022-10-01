use crate::parsing::data_table::ServerClass;
use crate::parsing::entities::Entity;
use hashbrown::HashMap;

pub fn extract_props(
    entities: &Option<HashMap<u32, Option<Entity>>>,
    props_names: &Vec<String>,
    tick: &i32,
    wanted_id: u32,
    wanted_steamid: u64,
    wanted_name: String,
    sv_cls_map: &HashMap<u16, ServerClass>,
) -> Vec<(String, f32)> {
    let mut tick_props: Vec<(String, f32)> = Vec::new();

    if entities.as_ref().unwrap().contains_key(&wanted_id) {
        if entities.as_ref().unwrap()[&wanted_id].is_some() {
            let ent = entities.as_ref().unwrap()[&wanted_id].as_ref().unwrap();

            for prop_name in props_names {
                if prop_name == "m_iClip1" {
                    let weapon_prop = parse_weapon_props(
                        entities,
                        wanted_id,
                        "m_hActiveWeapon".to_string(),
                        sv_cls_map,
                    );
                    tick_props.push((prop_name.to_string(), weapon_prop.1))
                } else {
                    if ent.props.contains_key(prop_name) {
                        tick_props
                            .push((prop_name.to_string(), ent.props[prop_name].data.to_float()))
                    } else {
                        tick_props.push((prop_name.to_string(), -1.0))
                    }
                }
            }

            tick_props.push(("tick".to_string(), *tick as f32));
            tick_props.push(("steamid".to_string(), wanted_steamid as f32));
            //tick_props.push(("name".to_string(), wanted_name));
        }
    }

    tick_props
}

fn parse_weapon_props(
    entities: &Option<HashMap<u32, Option<Entity>>>,
    wanted_id: u32,
    prop_name: String,
    sv_cls_map: &HashMap<u16, ServerClass>,
) -> (String, f32) {
    if entities.is_some() {
        if entities.as_ref().unwrap().contains_key(&wanted_id) {
            if entities.as_ref().unwrap()[&wanted_id].is_some() {
                let x = entities.as_ref().unwrap()[&wanted_id].as_ref().unwrap();

                if x.props.contains_key(&prop_name) {
                    let weapmask = x.props[&prop_name].data.to_float() as i32;
                    //println!("{}", weapmask & 0x7FF);
                    if entities
                        .as_ref()
                        .unwrap()
                        .contains_key(&((weapmask & 0x7FF) as u32))
                    {
                        let weap_ent = entities
                            .as_ref()
                            .unwrap()
                            .get(&((weapmask & 0x7FF) as u32))
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap();
                        let clip = weap_ent.props.get("m_iClip1");
                        match clip {
                            Some(c) => {
                                return (prop_name, c.data.to_float());
                            }
                            None => {}
                        }
                    }
                }
            }
        }
    }
    return (prop_name, -1.0);
}
