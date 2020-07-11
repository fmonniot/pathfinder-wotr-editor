// Data model for the save game
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Ref {
    #[serde(alias = "$ref")]
    reference: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Party {
    #[serde(alias = "$id")]
    id: String,
    #[serde(alias = "SceneName")]
    scene_name: String,
    #[serde(alias = "m_EntityData")]
    entities: Vec<Entity>,
    #[serde(alias = "HasEntityData")]
    has_entity_data: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entity {
    #[serde(alias = "Descriptor")]
    descriptor: Descriptor
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Descriptor {
    #[serde(alias = "Stats")]
    stats: HashMap<String, StatOrRef>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stat {
    #[serde(alias = "$id")]
    id: String,
    #[serde(alias = "Type")]
    tpe: String,
    #[serde(alias = "m_BaseValue")]
    base_value: i16
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum StatOrRef {
    Ref(Ref),
    Stat(Stat),
    #[serde(alias = "$id")]
    Id(String)
}

pub fn read_entity_from_path<P: AsRef<Path>>(path: P) -> Result<Entity, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(u)
}