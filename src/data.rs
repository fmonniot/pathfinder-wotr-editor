// Data model for the save game
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Party {
    json: IndexedJson,
    characters: Vec<Character>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    id: String,
    name: String,
    statistics: Vec<Stat>
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stat {
    #[serde(alias = "$id")]
    id: String,
    #[serde(alias = "Type")]
    tpe: String,
    #[serde(alias = "m_BaseValue")]
    base_value: i16
}

#[derive(Debug)]
pub enum JsonReaderError {
    ArrayExpected(String, String), // path and actual type
    ObjectExpected(String, String), // path and actual type
    InvalidReference(String, String), // path and $ref value
    InvalidPointer(String),
    Deserialization(serde_json::Error),
}

impl std::convert::From<serde_json::Error> for JsonReaderError {
    fn from(err: serde_json::Error) -> Self {
        JsonReaderError::Deserialization(err)
    }
}

pub fn read_party(json: IndexedJson) -> Result<Party, JsonReaderError> {

    unimplemented!()
}

pub fn read_all_stats(json: &IndexedJson, character_index: u8) -> Result<Vec<Stat>, JsonReaderError> {
    let pointer = format!("/m_EntityData/{}/Descriptor/Stats", character_index);
    let stats = json.pointer(&pointer).ok_or(JsonReaderError::InvalidPointer(pointer.to_string()))?;
    
    // TODO This is actually an object with the key being the attribute
    let stats = stats.as_object().ok_or(JsonReaderError::ObjectExpected(pointer.to_string(), json_type(stats).to_string()))?;

    let stats = stats.iter()
        .filter(|(key, _)| key != &"$id")
        .map(|(key, value)| {
            let ptr = format!("{}/{}", pointer, key);
            let value = dereference(value, json, &ptr)?;
            let stat = serde_json::from_value(value.clone())?;

            Ok(stat)
        })
        .collect::<Result<Vec<_>, JsonReaderError>>()?;

    Ok(stats)   
}

fn dereference<'a>(value: &'a Value, index: &'a IndexedJson, path: &str) -> Result<&'a Value, JsonReaderError> {

    let sta = value.as_object().ok_or(JsonReaderError::ObjectExpected(path.to_string(), json_type(value).to_string()))?;

    match sta.get("$ref").and_then(|j| j.as_str()) {
        Some(reference) => index.reference(reference).ok_or(JsonReaderError::InvalidReference(path.to_string(), reference.to_string())),
        None => Ok(value)
    }
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(..) => "bool",
        Value::Number(..) => "number",
        Value::String(..) => "string",
        Value::Array(..) => "array",
        Value::Object(..) => "object",
    }
}

pub fn read_entity_from_path<P: AsRef<Path>>(path: P) -> Result<Value, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file
    Ok(serde_json::from_reader(reader)?)
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedJson {
    json: Value,
    pub index: BTreeMap<String, String>
}

impl IndexedJson {
    pub fn new(json: Value) -> IndexedJson {
        let mut index = BTreeMap::new();
        build_index(&json, "", &mut index);

        IndexedJson { json, index }
    }

    // TODO Maybe opaque type for references ?
    pub fn reference(&self, reference: &str) -> Option<&Value> {
        self.index.get(reference)
            .and_then(|pointer| self.json.pointer(pointer))
    }

    pub fn pointer(&self, pointer: &str) -> Option<&Value> {
        self.json.pointer(pointer)
    }

    pub fn pointer_mut(&mut self, pointer: &str) -> Option<&mut Value> {
        self.json.pointer_mut(pointer)
    }
}

fn build_index(json: &Value, path: &str,
               index: &mut BTreeMap<String, String>) {
    match json {
        Value::Array(values) => {
            for (idx, value) in values.iter().enumerate() {
                build_index(value, &format!("{}/{}", path, idx), index);
            }
        },
        Value::Object(map) => {
            // Check if there is an $id. If so, add the id with the json pointer to the index
            map.get("$id")
                .and_then(|j| j.as_str())
                .and_then(|id| index.insert(id.to_string(), path.to_string()));

            for (key, value) in map {
                if key == "$id" { continue };

                build_index(value, &format!("{}/{}", path, key), index);
            }
        },
        _ => ()
    }
}