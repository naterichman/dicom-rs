use std::collections::HashMap;
use std::str::FromStr;

use dicom_core::dictionary::TagRange;
use serde::{Deserialize, Serialize};


fn load_standard() -> serde_cbor::Value{
    let sop_to_ciod = include_bytes!(concat!(env!("OUT_DIR"), "/CIOD_TO_MODULES.cbor"));
    let ciod_to_modules = include_bytes!(concat!(env!("OUT_DIR"), "/MODULE_TO_ATTRIBUTES.cbor"));
    let modules_to_attributes = include_bytes!(concat!(env!("OUT_DIR"), "/SOP_TO_CIOD.cbor"));

    let attributes: Attributes = serde_cbor::from_slice(modules_to_attributes).unwrap();
    let modules: Modules = serde_cbor::from_slice(ciod_to_modules).unwrap();
    serde_cbor::from_slice(standard).unwrap()
}

pub struct Standard {
    pub sop_to_ciod: HashMap<String, String>,
    pub ciod_to_modules: Modules,
    pub modules_to_attributes: Attributes,
}

#[derive(Deserialize)]
pub struct Modules(HashMap<String, Vec<Module>>);

#[derive(Debug, Deserialize)]
pub struct Module {
    pub module_id: String,
    pub usage: String,
    pub conditional: Option<String>,
    pub ie: String
}

impl Module {
    pub fn is_required(&self) -> bool {
        if &self.usage == "M" {
            return true;
        }
        if self.conditional.is_some() && self.usage == "C" {
            unimplemented!("Conditional modules are not supported yet")
        }
        false
    }
}

fn deserialize_tag_range<'de, D>(
    deserializer: D,
) -> Result<TagRange, D::Error> 
where D: serde::Deserializer<'de> {
    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(TagRange::from_str(&s))
}

#[derive(Deserialize)]
pub struct Attributes(HashMap<String, Vec<Attribute>>);

#[derive(Deserialize)]
pub struct Attribute {

    #[serde(deserialize_with = "deserialize_tag_range")]
    pub tag: TagRange,
    #[serde(deserialize_with = "deserialize_tag_type")]
    pub tag_type: TagType
}

fn deserialize_tag_type<'de, D>(
    deserializer: D,
) -> Result<TagType, D::Error> 
where D: serde::Deserializer<'de> {
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s {
        "1" => Ok(TagType::Type1),
        "1C" => Ok(TagType::Type1C),
        "2" => Ok(TagType::Type2),
        "2C" => Ok(TagType::Type2C),
        "3" => Ok(TagType::Type3),
        _ => Ok(TagType::Unknown)
    }
}

#[derive(Deserialize)]
pub enum TagType {
    Type1,
    Type1C,
    Type2,
    Type2C,
    Type3,
    Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_standard() {
        let standard = load_standard();
    }
}