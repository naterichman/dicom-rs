use std::collections::{HashMap, BTreeMap};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read, BufWriter, Write};
use std::path::Path;
use serde_cbor::Value;

macro_rules! warning {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn load_json(path: &Path) -> serde_json::Value {
    let mut file = BufReader::new(File::open(path).unwrap());
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    serde_json::from_str(&content).unwrap()
}


fn handle_sop_coid_map(sops: serde_json::Value, ciods: serde_json::Value, out_path: &Path){
    let path = out_path.join("SOP_TO_CIOD.cbor");
    warning!("Writing to {:?}", path);
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    let sop_to_ciod_name: HashMap<&str, &str> = sops.as_array().unwrap().iter().map(|sop| {
        let sop = sop.as_object().unwrap();
        (sop["id"].as_str().unwrap(), sop["ciod"].as_str().unwrap())
    }).collect();

    let ciod_name_to_id: HashMap<&str, &str> = ciods.as_array().unwrap().iter().map(|ciod| {
        let ciod = ciod.as_object().unwrap();
        (ciod["name"].as_str().unwrap(), ciod["id"].as_str().unwrap())
    }).collect();

    let sop_to_ciod_id = sop_to_ciod_name.iter().filter_map(|(sop, ciod_name)| {
        ciod_name_to_id.get(ciod_name).map(|ciod_id| 
            (Value::Text((*sop).to_string()), Value::Text((*ciod_id).to_string()))
        )
    }).collect::<BTreeMap<_, _>>();

    let value = serde_cbor::Value::Map(sop_to_ciod_id);
    serde_cbor::to_writer(&mut writer, &value).unwrap();
}

fn handle_cid_module_map(modules: serde_json::Value, out_path: &Path){
    let path = out_path.join("CIOD_TO_MODULES.cbor");
    warning!("Writing to {:?}", path);
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    let mut ciod_module_map: BTreeMap<Value, Value> = BTreeMap::new();
    let to_str_value = |inner: Option<&serde_json::Value>|{
        inner.map_or(
            Value::Null,
            |v| Value::Text(v.as_str().unwrap_or("").to_string()),
        )
    };
    modules.as_array().unwrap().iter()
        .for_each(|module| {
            let module = module.as_object().unwrap();
            let ciod = to_str_value(module.get("ciodId"));
            if let Value::Array(inner) = ciod_module_map.entry(ciod)
                .or_insert(Value::Array(vec![])){
                    inner.push(Value::Map(BTreeMap::from([
                            ("moduleId".to_string(), module.get("moduleId")),
                            ("usage".to_string(), module.get("usage")),
                            ("conditional".to_string(), module.get("conditionalStatement")),
                            ("ie".to_string(), module.get("informationEntity"))
                        ].iter().map(|(k, v)| 
                            (Value::Text(k.to_string()), to_str_value(*v))
                        )
                        .collect::<BTreeMap<_, _>>()
                    ))
                );
            }
        });
    serde_cbor::to_writer(&mut writer, &serde_cbor::Value::Map(ciod_module_map)).unwrap();

}


fn handle_module_attribute_map(attributes: serde_json::Value, out_path: &Path){
    let path = out_path.join("MODULE_TO_ATTRIBUTES.cbor");
    warning!("Writing to {:?}", path);
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    let mut module_attribute_map: BTreeMap<Value, Value> = BTreeMap::new();
    warning!("{:?}", attributes.as_array().unwrap().len());
    let to_str_value = |inner: Option<&serde_json::Value>|{
        inner.map_or(
            Value::Null,
            |v| Value::Text(v.as_str().unwrap_or("").to_string()),
        )
    };
    attributes.as_array().unwrap().iter()
        .for_each(|attribute|{
            let attribute = attribute.as_object().unwrap();
            if let Value::Array(inner) = module_attribute_map.entry(to_str_value(attribute.get("moduleId")))
                .or_insert(Value::Array(vec![])){
                    inner.push(Value::Map(BTreeMap::from([
                        ("tag", attribute.get("tag")),
                        ("type", attribute.get("type"))
                        ].iter().map(|(k, v)| 
                            (Value::Text(k.to_string()), to_str_value(*v))
                        )
                        .collect::<BTreeMap<_, _>>()
                    )));
                }
        });
    serde_cbor::to_writer(&mut writer, &serde_cbor::Value::Map(module_attribute_map)).unwrap();
}

fn main() {
    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("dicom-standard/standard/");

    let sops = load_json(&path.join("sops.json"));
    let ciods = load_json(&path.join("ciods.json"));
    let modules = load_json(&path.join("ciod_to_modules.json"));
    let attributes = load_json(&path.join("module_to_attributes.json"));
    let path = Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf();


    handle_sop_coid_map(sops, ciods, &path);
    handle_cid_module_map(modules, &path);
    handle_module_attribute_map(attributes, &path)
   // writeln!(
    //    &mut writer,
    //    "use dicom_core::{{Tag, dictionary::TagRange}};"
    //).unwrap();
    //serde_cbor::to_writer(&mut writer, &sops).unwrap();
}