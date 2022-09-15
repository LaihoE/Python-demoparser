use std::collections::HashMap;

fn main() {
    let mut x: HashMap<String, f32> = HashMap::new();
    let mut y: HashMap<String, f32> = HashMap::new();

    x.insert("x".to_string(), 0.0);
    y.insert("y".to_string(), 42.0);

    x.extend(y);
    println!("{:?}", &x);
}
