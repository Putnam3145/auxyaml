use yaml_rust::{Yaml, YamlEmitter};

use linked_hash_map::LinkedHashMap;

use auxtools::{hook, List, Value, runtime};

fn yaml_to_value(yaml: &Yaml) -> Value {
    match yaml {
        Yaml::Real(y) => Value::from(y.parse::<f32>().unwrap_or_default()),
        Yaml::Integer(y) => Value::from(*y as f32),
        Yaml::String(y) => Value::from_string(y).unwrap(),
        Yaml::Boolean(y) => Value::from(*y),
        Yaml::Array(y) => Value::from(List::from_iter(y.iter().map(yaml_to_value))),
        Yaml::Hash(y) => {
            let l = List::new();
            for (k,v) in y.iter() {
                l.set(yaml_to_value(k), yaml_to_value(v)).unwrap();
            }
            Value::from(l)
        }
        Yaml::Alias(y) => Value::from(*y as f32),
        _ => Value::null(),
    }
}

fn value_to_yaml(value: &Value) -> Yaml {
    if let Ok(l) = value.as_list() {
        // gotta make both because auxtools gives us no way to know whether it's associative or not lol
        let mut vec: Vec<Yaml> = Vec::with_capacity(l.len() as usize);
        let mut map = LinkedHashMap::with_capacity(l.len() as usize);
        let mut is_assoc = false;
        for i in 1..=l.len() {
            let k = l.get(i).unwrap();
            let v = l.get(k.clone());
            is_assoc = is_assoc || v.is_ok();
            if !is_assoc {
                vec.push(value_to_yaml(&k));
            }
            map.insert(value_to_yaml(&k),value_to_yaml(&v.unwrap_or(Value::null())));
        }
        if is_assoc {
            return Yaml::Array(vec);
        } else {
            return Yaml::Hash(map);
        }
    } else {
        if let Ok(v) = value.as_number() {
            return Yaml::Real(v.to_string());
        }
        if let Ok(v) = value.as_string() {
            return Yaml::String(v);
        }
    }
    Yaml::Null
}

#[hook("/proc/yaml_decode")]
fn decode_yaml(yaml_v: Value) {
    Ok(yaml_to_value(&Yaml::from_str(&yaml_v.as_string()?)))
}

#[hook("/proc/yaml_encode")]
fn encode_yaml(v: Value) {
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(&value_to_yaml(v)).map_err(|_| runtime!("Conversion of Byond value to YAML failed!"))?;
    Value::from_string(out_str)
}