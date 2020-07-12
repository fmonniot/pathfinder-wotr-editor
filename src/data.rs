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
    index: IndexedJson,
    pub characters: Vec<Character>,
}

/* TODO Aligment
   Aligment is found in `/Descriptor/Alignment/m_History/<last>/Position`:

       "Position": {
            "x": -4.28543032E-08,
            "y": 1.0
        }

    `x` is describing the lawful/chaotic axis and `y` the good/evil one.
    Needs to find out when the neutral switch happens but it looks like that
    follow the disc-shape layout of the game (See samples/pfkm_alignment_wheels
    for a visual representation)
*/
#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub blueprint: String,
    pub experience: u64,
    pub mythic_experience: u64,
    pub statistics: Vec<Stat>,
}

impl Character {
    pub fn find_stat(&self, name: &str) -> Option<&Stat> {
        self.statistics.iter().find(|s| s.tpe == name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stat {
    #[serde(alias = "$id")]
    pub id: String,
    #[serde(alias = "Type")]
    pub tpe: String,
    #[serde(alias = "m_BaseValue")]
    pub base_value: u64,
}

#[derive(Debug)]
pub enum JsonReaderError {
    ArrayExpected(String, String),  // path and actual type
    ObjectExpected(String, String), // path and actual type
    StringExpected(String, String), // path and actual type
    NumberExpected(String, String), // path and actual type

    InvalidReference(String, String), // path and $ref value
    InvalidPointer(String),
    Deserialization(serde_json::Error),
}

impl std::convert::From<serde_json::Error> for JsonReaderError {
    fn from(err: serde_json::Error) -> Self {
        JsonReaderError::Deserialization(err)
    }
}

pub fn read_party(index: IndexedJson) -> Result<Party, JsonReaderError> {
    let pointer = "/m_EntityData";

    let characters_json = index
        .pointer(&pointer)
        .ok_or(JsonReaderError::InvalidPointer(pointer.to_string()))?;
    let characters_json = characters_json
        .as_array()
        .ok_or(JsonReaderError::ArrayExpected(
            pointer.to_string(),
            json_type(characters_json).to_string(),
        ))?;

    let characters = characters_json
        .iter()
        .filter(|json| {
            // Only keep the entry of type unit
            json.get("$type")
                .and_then(|j| j.as_str())
                .filter(|s| s == &"Kingmaker.EntitySystem.Entities.UnitEntityData, Assembly-CSharp")
                .is_some()
        })
        .map(|json| read_character(&index, json))
        .collect::<Result<Vec<_>, JsonReaderError>>()?;

    Ok(Party { index, characters })
}

fn read_character(index: &IndexedJson, json: &Value) -> Result<Character, JsonReaderError> {
    // Statistics
    let pointer = "/Descriptor/Stats";

    let stats_json = json
        .pointer(&pointer)
        .ok_or(JsonReaderError::InvalidPointer(pointer.to_string()))?;
    let stats_json = stats_json
        .as_object()
        .ok_or(JsonReaderError::ObjectExpected(
            pointer.to_string(),
            json_type(stats_json).to_string(),
        ))?;

    let statistics = stats_json
        .iter()
        .filter(|(key, _)| key != &"$id")
        .map(|(key, value)| {
            let ptr = format!("{}/{}", pointer, key);
            let value = dereference(value, &index, &ptr)?;
            let stat = serde_json::from_value(value.clone())?;

            Ok(stat)
        })
        .collect::<Result<Vec<_>, JsonReaderError>>()?;

    // Better error type ?
    let id = json
        .get("$id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or(JsonReaderError::InvalidPointer("/$id".to_string()))?;
    let name = json
        .pointer("/Descriptor/CustomName")
        .ok_or(JsonReaderError::InvalidPointer(
            "/Descriptor/CustomName".to_string(),
        ))?
        .as_str()
        .ok_or(JsonReaderError::StringExpected(
            pointer.to_string(),
            "todo".to_string(),
        ))?
        .to_string();

    let blueprint = json
        .pointer("/Descriptor/Blueprint")
        .ok_or(JsonReaderError::InvalidPointer(
            "/Descriptor/Blueprint".to_string(),
        ))?
        .as_str()
        .ok_or(JsonReaderError::StringExpected(
            pointer.to_string(),
            "todo".to_string(),
        ))?
        .to_string();

    let experience = json
        .pointer("/Descriptor/Progression/Experience")
        .ok_or(JsonReaderError::InvalidPointer(
            "/Descriptor/Blueprint".to_string(),
        ))?
        .as_u64()
        .ok_or(JsonReaderError::NumberExpected(
            "/Descriptor/Progression/Experience".to_string(),
            "todo".to_string(),
        ))?;

    let mythic_experience = json
        .pointer("/Descriptor/Progression/MythicExperience")
        .ok_or(JsonReaderError::InvalidPointer(
            "/Descriptor/Blueprint".to_string(),
        ))?
        .as_u64()
        .ok_or(JsonReaderError::NumberExpected(
            "/Descriptor/Progression/MythicExperience".to_string(),
            "todo".to_string(),
        ))?;

    Ok(Character {
        id,
        name,
        blueprint,
        experience,
        mythic_experience,
        statistics,
    })
}

fn dereference<'a>(
    value: &'a Value,
    index: &'a IndexedJson,
    path: &str,
) -> Result<&'a Value, JsonReaderError> {
    let sta = value.as_object().ok_or(JsonReaderError::ObjectExpected(
        path.to_string(),
        json_type(value).to_string(),
    ))?;

    match sta.get("$ref").and_then(|j| j.as_str()) {
        Some(reference) => index
            .reference(reference)
            .ok_or(JsonReaderError::InvalidReference(
                path.to_string(),
                reference.to_string(),
            )),
        None => Ok(value),
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
    pub index: BTreeMap<String, String>,
}

impl IndexedJson {
    pub fn new(json: Value) -> IndexedJson {
        let mut index = BTreeMap::new();
        build_index(&json, "", &mut index);

        IndexedJson { json, index }
    }

    // TODO Maybe opaque type for references ?
    pub fn reference(&self, reference: &str) -> Option<&Value> {
        self.index
            .get(reference)
            .and_then(|pointer| self.json.pointer(pointer))
    }

    pub fn pointer(&self, pointer: &str) -> Option<&Value> {
        self.json.pointer(pointer)
    }
}

fn build_index(json: &Value, path: &str, index: &mut BTreeMap<String, String>) {
    match json {
        Value::Array(values) => {
            for (idx, value) in values.iter().enumerate() {
                build_index(value, &format!("{}/{}", path, idx), index);
            }
        }
        Value::Object(map) => {
            // Check if there is an $id. If so, add the id with the json pointer to the index
            map.get("$id")
                .and_then(|j| j.as_str())
                .and_then(|id| index.insert(id.to_string(), path.to_string()));

            for (key, value) in map {
                if key == "$id" {
                    continue;
                };

                build_index(value, &format!("{}/{}", path, key), index);
            }
        }
        _ => (),
    }
}
