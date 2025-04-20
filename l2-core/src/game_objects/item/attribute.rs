use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeSet {
    attributes: HashMap<Attribute, u16>
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Attribute {
    #[serde(rename = "")]
    None,
    Fire,
    Water,
    Earth,
    Wind,
    Dark,
    Unholy,
}

impl AttributeSet {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new()
        }
    }

    pub fn add(&mut self, attr: Attribute, value: u16) -> Result<(), &'static str> {
        match attr {
            Attribute::None => Ok(()),
            Attribute::Fire if self.attributes.contains_key(&Attribute::Water) => {
                Err("Cannot have both Fire and Water attributes")
            }
            Attribute::Water if self.attributes.contains_key(&Attribute::Fire) => {
                Err("Cannot have both Water and Fire attributes")
            }
            Attribute::Earth if self.attributes.contains_key(&Attribute::Wind) => {
                Err("Cannot have both Earth and Wind attributes")
            }
            Attribute::Wind if self.attributes.contains_key(&Attribute::Earth) => {
                Err("Cannot have both Wind and Earth attributes")
            }
            Attribute::Dark if self.attributes.contains_key(&Attribute::Unholy) => {
                Err("Cannot have both Dark and Unholy attributes")
            }
            Attribute::Unholy if self.attributes.contains_key(&Attribute::Dark) => {
                Err("Cannot have both Unholy and Dark attributes")
            }
            _ => {
                self.attributes.insert(attr, value);
                Ok(())
            }
        }
    }
}

impl Serialize for AttributeSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.attributes.len()))?;
        for (key, value) in &self.attributes {
            match key {
                Attribute::None => continue,
                _ => map.serialize_entry(&key, &value)?
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AttributeSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: HashMap<Attribute, u16> = HashMap::deserialize(deserializer)?;
        let mut attr_set = AttributeSet::new();
        
        for (attr, value) in map {
            if let Err(e) = attr_set.add(attr, value) {
                return Err(serde::de::Error::custom(e));
            }
        }
        
        Ok(attr_set)
    }
}