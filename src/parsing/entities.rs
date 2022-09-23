use super::data_table::server_class_blueprint;
use crate::parsing::data_table::ServerClass;
use crate::parsing::game_events::HurtEvent;
use crate::parsing::newbitreader::Bitr;
use crate::parsing::read_bits::BitReader;
use crate::parsing::read_bits::PropAtom;
use crate::parsing::read_bits::PropData;
use crate::Demo;
use csgoproto::netmessages::csvcmsg_send_table::Sendprop_t;
use csgoproto::netmessages::CSVCMsg_PacketEntities;
use csgoproto::netmessages::CSVCMsg_SendTable;
use fxhash::FxHashMap;
use hashbrown::HashMap;
use protobuf;
use protobuf::Message;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::vec;

#[derive(Debug)]
pub struct Entity {
    pub class_id: u32,
    pub entity_id: u32,
    pub serial: u32,
    pub props: HashMap<String, PropAtom>,
}

#[derive(Debug)]
pub struct Prop {
    pub prop: Sendprop_t,
    pub arr: Option<Sendprop_t>,
    pub table: CSVCMsg_SendTable,
    pub col: i32,
    pub data: Option<PropData>,
}

impl Demo {
    pub fn parse_packet_entities(
        pack_ents: CSVCMsg_PacketEntities,
        should_parse: bool,
        class_bits: u32,
        entities: Arc<Mutex<HashMap<u32, Option<Entity>>>>,
        dt_map: Arc<Mutex<Option<HashMap<String, CSVCMsg_SendTable>>>>,
        tick: i32,
        serverclass_map: Arc<Mutex<HashMap<u16, ServerClass>>>,
    ) {
        // println!("INSIDE");
        // println!("HERERERE");
        // Vec<(u32, Option<Entity>)>
        let mut new_ents = vec![];
        let n_upd_ents = pack_ents.updated_entries();
        let left_over = (pack_ents.entity_data().len() % 4) as i32;
        let mut b = BitReader::new(pack_ents.entity_data(), left_over);
        b.read_uneven_end_bits();
        let mut entity_id: i32 = -1;

        for xx in 0..n_upd_ents {
            //println!("CNT {}", n_upd_ents);
            let sc_map_clone = Arc::clone(&serverclass_map);
            let dt_map_clone = Arc::clone(&dt_map);
            let entities_clone = Arc::clone(&entities);
            let entplus: i32 = b.read_u_bit_var().try_into().unwrap();
            //println!("ENTPLUS {}", entplus);

            entity_id += 1 + (entplus);

            if b.read_bool() {
                b.read_bool();
            } else if b.read_bool() {
                let cls_id = b.read_nbits(class_bits as usize);
                let serial = b.read_nbits(10);

                let e = Entity {
                    class_id: cls_id,
                    entity_id: entity_id.try_into().unwrap(),
                    serial: serial,
                    props: HashMap::default(),
                };
                new_ents.push((entity_id.try_into().unwrap(), Some(e)));
                let mut e = Entity {
                    class_id: cls_id,
                    entity_id: entity_id.try_into().unwrap(),
                    serial: serial,
                    props: HashMap::default(),
                };

                let data = Demo::read_new_ent(&e, &mut b, tick, sc_map_clone);
                //println!("DATA LEN {}", data.len());
            } else {
                loop {
                    let hm = entities_clone.lock().unwrap();
                    let ent = hm.get(&(entity_id.try_into().unwrap()));
                    match ent {
                        Some(x) => match x {
                            Some(x) => {
                                let ent = x;
                                let data = Demo::read_new_ent(ent, &mut b, tick, sc_map_clone);
                                let e = Entity {
                                    class_id: x.class_id,
                                    entity_id: x.entity_id.try_into().unwrap(),
                                    serial: x.serial,
                                    props: HashMap::default(),
                                };
                                new_ents.push((e.entity_id.try_into().unwrap(), Some(e)));
                                break;
                            }
                            None => {
                                drop(ent);
                                drop(hm);
                            }
                        },
                        None => {
                            drop(ent);
                            drop(hm);
                            /*
                            let ten_millis = time::Duration::from_nanos(10);
                            let now = time::Instant::now();
                            thread::sleep(ten_millis);
                            */
                        }
                        _ => {
                            panic!("WTF")
                        }
                    }
                }
            }
        }

        entities.lock().unwrap().extend(new_ents);
        drop(entities);
        if tick % 100 == 0 {
            println!("{}", tick);
        }

        //new_ents
    }

    pub fn handle_entity_upd(
        sv_cls: &ServerClass,
        b: &mut BitReader<&[u8]>,
        tick: i32,
    ) -> Vec<PropAtom> {
        let mut val = -1;
        let new_way = b.read_bool();
        let mut indicies = vec![];

        loop {
            val = b.read_inx(val, new_way);
            if val == -1 {
                break;
            }
            indicies.push(val);
        }

        let mut props: Vec<PropAtom> = Vec::with_capacity(indicies.len());

        for inx in indicies {
            let prop = &sv_cls.fprops.as_ref().unwrap()[inx as usize];
            let pdata = b.decode(prop);
            //println!("{:?}", tick);

            match pdata {
                PropData::VecXY(v) => {
                    let endings = ["_X", "_Y"];
                    for inx in 0..2 {
                        let data = PropData::F32(v[inx]);
                        let name = prop.prop.var_name().to_string() + endings[inx];
                        let atom = PropAtom {
                            prop_name: name,
                            data: data,
                            tick: tick,
                        };
                        props.push(atom);
                    }
                }
                PropData::VecXYZ(v) => {
                    let endings = ["_X", "_Y", "_Z"];
                    for inx in 0..3 {
                        let data = PropData::F32(v[inx]);
                        let name = prop.prop.var_name().to_string() + endings[inx];
                        let atom = PropAtom {
                            prop_name: name,
                            data: data,
                            tick: tick,
                        };
                        props.push(atom);
                    }
                }

                PropData::String(_) => {}
                _ => {
                    let atom = PropAtom {
                        prop_name: prop.prop.var_name().to_string(),
                        data: pdata,
                        tick: tick,
                    };
                    props.push(atom);
                }
            }
        }
        props
    }

    pub fn read_new_ent(
        ent: &Entity,
        b: &mut BitReader<&[u8]>,
        tick: i32,
        serverclass_map: Arc<Mutex<HashMap<u16, ServerClass>>>,
    ) -> Vec<PropAtom> {
        let mut data = vec![];
        let mut cnt = 0;
        loop {
            cnt += 1;

            let mtx = &serverclass_map.lock().unwrap();
            let sv_cls_opt = mtx.get(&(ent.class_id as u16));
            match sv_cls_opt {
                Some(svcls) => {
                    let props = Demo::handle_entity_upd(svcls, b, tick);

                    data.extend(props);
                    drop(mtx);
                    drop(sv_cls_opt);

                    break;
                }
                None => {
                    // MAYBE ENT CHANGE MID PARSING

                    drop(mtx);
                    drop(sv_cls_opt);
                    /*
                    let ten_millis = time::Duration::from_nanos(10);
                    let now = time::Instant::now();
                    thread::sleep(ten_millis);
                    */
                }
                _ => {
                    panic!("WTF");
                }
            }
        }

        data
    }

    pub fn get_excl_props(
        table: CSVCMsg_SendTable,
        dt_map: Arc<Mutex<Option<HashMap<String, CSVCMsg_SendTable>>>>,
    ) -> Vec<Sendprop_t> {
        let mut excl = vec![];
        let mut cnt = 0;
        for prop in &table.props {
            cnt += 1;
            let cloned_dt_map = Arc::clone(&dt_map);
            if prop.flags() & (1 << 6) != 0 {
                excl.push(prop.clone());
            }
            if prop.type_() == 6 {
                let sub_table =
                    cloned_dt_map.lock().unwrap().as_ref().unwrap()[prop.dt_name()].clone();
                excl.extend(Demo::get_excl_props(sub_table, cloned_dt_map));
            }
        }
        excl
    }
    // Vec<Prop<'a>>
    pub fn flatten_dt(
        table: CSVCMsg_SendTable,
        dt_map: Arc<Mutex<Option<HashMap<String, CSVCMsg_SendTable>>>>,
    ) -> Vec<Prop> {
        let temp = Arc::clone(&dt_map);
        let excl = Demo::get_excl_props(table.clone(), dt_map);
        let mut newp = Demo::get_props(table, &excl, temp);
        let mut prios = vec![];
        for p in &newp {
            prios.push(p.prop.priority());
        }

        let set: HashSet<_> = prios.drain(..).collect();
        prios.extend(set.into_iter());
        prios.push(64);
        prios.sort();
        let mut start = 0;

        for prio_inx in 0..prios.len() {
            let priority = prios[prio_inx];
            loop {
                let mut currentprop = start;
                while currentprop < newp.len() {
                    let prop = newp[currentprop].prop.clone();
                    if prop.priority() == priority
                        || priority == 64 && ((prop.flags() & (1 << 18)) != 0)
                    {
                        if start != currentprop {
                            newp.swap(start, currentprop);
                        }
                        start += 1;
                    }
                    currentprop += 1;
                }
                if currentprop == newp.len() {
                    break;
                }
            }
        }
        newp
    }

    pub fn flatten_dt_data_table(
        table: CSVCMsg_SendTable,
        dt_map: Arc<Mutex<Option<HashMap<String, CSVCMsg_SendTable>>>>,
        blueprint: server_class_blueprint,
        serverclass_map_clone: Arc<Mutex<HashMap<u16, ServerClass>>>,
    ) {
        let temp = Arc::clone(&dt_map);
        let excl = Demo::get_excl_props(table.clone(), dt_map);
        let mut newp = Demo::get_props(table, &excl, temp);
        let mut prios = vec![];
        for p in &newp {
            prios.push(p.prop.priority());
        }

        let set: HashSet<_> = prios.drain(..).collect();
        prios.extend(set.into_iter());
        prios.push(64);
        prios.sort();
        let mut start = 0;

        for prio_inx in 0..prios.len() {
            let priority = prios[prio_inx];
            loop {
                let mut currentprop = start;
                while currentprop < newp.len() {
                    let prop = newp[currentprop].prop.clone();
                    if prop.priority() == priority
                        || priority == 64 && ((prop.flags() & (1 << 18)) != 0)
                    {
                        if start != currentprop {
                            newp.swap(start, currentprop);
                        }
                        start += 1;
                    }
                    currentprop += 1;
                }
                if currentprop == newp.len() {
                    break;
                }
            }
        }

        let server_class = ServerClass {
            id: blueprint.id,
            name: blueprint.name.clone(),
            dt: blueprint.dt.clone(),
            fprops: Some(newp),
        };
        serverclass_map_clone
            .lock()
            .unwrap()
            .insert(server_class.id, server_class);
    }

    #[inline]
    pub fn is_prop_excl(excl: Vec<Sendprop_t>, table: CSVCMsg_SendTable, prop: Sendprop_t) -> bool {
        for item in excl {
            if table.net_table_name() == item.dt_name() && prop.var_name() == item.var_name() {
                return true;
            }
        }
        false
    }

    pub fn get_props(
        table: CSVCMsg_SendTable,
        excl: &Vec<Sendprop_t>,
        dt_map: Arc<Mutex<Option<HashMap<String, CSVCMsg_SendTable>>>>,
    ) -> Vec<Prop> {
        let mut flat: Vec<Prop> = Vec::new();
        let mut child_props = Vec::new();
        let mut cnt = 0;

        for prop in &table.props {
            let dt_map_clone = Arc::clone(&dt_map);
            if (prop.flags() & (1 << 8) != 0)
                || (prop.flags() & (1 << 6) != 0)
                || Demo::is_prop_excl(excl.to_vec(), table.clone(), prop.clone())
            {
                continue;
            }

            if prop.type_() == 6 {
                let sub_table = dt_map.lock().unwrap().as_ref().unwrap()[prop.dt_name()].clone();
                child_props = Demo::get_props(sub_table.clone(), excl, dt_map_clone);

                if (prop.flags() & (1 << 11)) == 0 {
                    for mut p in child_props {
                        p.col = 0;
                        flat.push(p);
                    }
                } else {
                    for mut p in child_props {
                        flat.push(p);
                    }
                }
            } else if prop.type_() == 5 {
                let prop_arr = Prop {
                    prop: prop.clone(),
                    arr: Some(table.props[cnt].clone()),
                    table: table.clone(),
                    col: 1,
                    data: None,
                };
                flat.push(prop_arr);
            } else {
                let prop = Prop {
                    prop: prop.clone(),
                    arr: None,
                    table: table.clone(),
                    col: 1,
                    data: None,
                };
                flat.push(prop);
            }
            cnt += 1;
        }
        flat.sort_by_key(|x| x.col);
        return flat;
    }
}
