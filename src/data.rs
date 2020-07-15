/// Data model for the save game
/// TODO Improve the pointer and conversion logic. At the moment we have a lot of repetition
/// to manage errors.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Party {
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

    pub fn name(&self) -> String {
        if self.name.is_empty() {
            Character::blueprint_to_name(&self.blueprint).unwrap_or_else(|| self.blueprint.clone())
        } else {
            self.name.clone()
        }
    }

    fn blueprint_to_name(s: &str) -> Option<String> {
        let opt = match s {
            "397b090721c41044ea3220445300e1b8" => Some("Camellia"),
            "54be53f0b35bf3c4592a97ae335fe765" => Some("Seelah"),
            "cb29621d99b902e4da6f5d232352fbda" => Some("Laan"),
            "766435873b1361c4287c351de194e5f9" => Some("Woljif"),
            "2779754eecffd044fbd4842dba55312c" => Some("Ember"),
            "096fc4a96d675bb45a0396bcaa7aa993" => Some("Daeran"),
            "8a6986e17799d7d4b90f0c158b31c5b9" => Some("Smilodon"), // Or pet in general ?
            "1cbbbb892f93c3d439f8417ad7cbb6aa" => Some("Sosiel"),
            "f72bb7c48bb3e45458f866045448fb58" => None, // Unknown at the moment, let me progress in game. maybe.
            _ => None,
        };

        opt.map(str::to_string)
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

    InvalidReference(String, String), // path and $ref value
    InvalidPointer(String),           // pointer
    Deserialization(serde_json::Error),
}

impl std::convert::From<serde_json::Error> for JsonReaderError {
    fn from(err: serde_json::Error) -> Self {
        JsonReaderError::Deserialization(err)
    }
}

pub fn read_party(index: &IndexedJson) -> Result<Party, JsonReaderError> {
    let characters = reader::pointer_as_array(&index.json, "/m_EntityData")?
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

    Ok(Party { characters })
}

fn read_character(index: &IndexedJson, json: &Value) -> Result<Character, JsonReaderError> {
    let statistics = reader::pointer_as_object(&json, "/Descriptor/Stats")?
        .iter()
        .filter(|(key, _)| key != &"$id")
        .map(|(key, value)| {
            let ptr = format!("/Descriptor/Stats/{}", key);
            let value = index.dereference(value, &ptr)?;
            let stat = serde_json::from_value(value.clone())?;

            Ok(stat)
        })
        .collect::<Result<Vec<_>, JsonReaderError>>()?;

    let id = reader::pointer_as(&json, "/$id")?;
    let name = reader::pointer_as(&json, "/Descriptor/CustomName")?;
    let blueprint = reader::pointer_as(&json, "/Descriptor/Blueprint")?;
    let experience = reader::pointer_as(&json, "/Descriptor/Progression/Experience")?;
    let mythic_experience = reader::pointer_as(&json, "/Descriptor/Progression/MythicExperience")?;

    Ok(Character {
        id,
        name,
        blueprint,
        experience,
        mythic_experience,
        statistics,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub armies: Vec<Army>,
    pub money: u64,
    pub recruits: RecruitsManager,
    pub resources: KingdomResources,
    pub resources_per_turn: KingdomResources,
    // TODO modifiers (rankup, claim, etc…)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Army {
    id: String,
    experience: u64,
    movement_points: f64,
    squads: Vec<Squad>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Squad {
    #[serde(alias = "$id")]
    id: String,
    #[serde(alias = "Unit")]
    unit: String,
    #[serde(alias = "Count")]
    count: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RecruitsManager {
    #[serde(alias = "m_Pool")]
    pool: Vec<Recruit>,
    #[serde(alias = "m_Growth")]
    growth: Vec<Recruit>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Recruit {
    #[serde(alias = "$id")]
    id: String,
    #[serde(alias = "Unit")]
    unit: String,
    #[serde(alias = "Count")]
    count: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct KingdomResources {
    #[serde(alias = "$id")]
    id: String,
    #[serde(alias = "m_Finances")]
    pub finances: u64,
    #[serde(alias = "m_Basics")]
    pub basics: u64,
    #[serde(alias = "m_Favors")]
    pub favors: u64,
    #[serde(alias = "m_Mana")]
    pub mana: u64,
}

pub fn read_player(index: &IndexedJson) -> Result<Player, JsonReaderError> {
    let armies = reader::pointer_as_array(&index.json, "/m_GlobalMaps")?
        .iter()
        .map(|json| {
            reader::pointer_as_array(&json, "/m_Armies")?
                .iter()
                .filter(|json| {
                    // We only keep the crusaders squads
                    json.pointer("/Data/Faction")
                        .and_then(|v| v.as_str())
                        .filter(|s| s == &"Crusaders")
                        .is_some()
                })
                .map(|json| {
                    let id = reader::pointer_as(&json, "/$id")?;
                    let movement_points = reader::pointer_as(&json, "/MovementPoints")?;
                    let experience = reader::pointer_as(&json, "/Data/Experience")?;
                    let squads = reader::pointer_as(&json, "/Data/Squads")?;

                    Ok(Army {
                        id,
                        experience,
                        movement_points,
                        squads,
                    })
                })
                .collect::<Result<Vec<_>, JsonReaderError>>()
        })
        .collect::<Result<Vec<_>, JsonReaderError>>()?
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    let resources = reader::pointer_as(&index.json, "/Kingdom/Resources")?;
    let resources_per_turn = reader::pointer_as(&index.json, "/Kingdom/ResourcesPerTurn")?;
    let recruits = reader::pointer_as(&index.json, "/Kingdom/RecruitsManager")?;
    let money = reader::pointer_as(&index.json, "/Money")?;

    Ok(Player {
        armies,
        money,
        recruits,
        resources,
        resources_per_turn,
    })
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

    /// Get the value following a JSON pointer `path`. If the pointed node is a JSON
    /// object containing the field `$ref`, return the JSON node with the associated
    /// `$id`.
    fn dereference<'a>(
        &'a self,
        value: &'a Value,
        path: &str,
    ) -> Result<&'a Value, JsonReaderError> {
        let sta = value.as_object().ok_or_else(|| {
            JsonReaderError::ObjectExpected(path.to_string(), reader::json_type(value).to_string())
        })?;

        match sta.get("$ref").and_then(|j| j.as_str()) {
            Some(reference) => self
                .index
                .get(reference)
                .and_then(|pointer| self.json.pointer(pointer))
                .ok_or_else(|| {
                    JsonReaderError::InvalidReference(path.to_string(), reference.to_string())
                }),
            None => Ok(value),
        }
    }

    /* When we start doing actual modification
    pub fn pointer_mut(&mut self, pointer: &str) -> Option<&mut Value> {
        self.json.pointer_mut(pointer)
    }
    */
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

/// A module containing helper functions to read data from a [serde_json::Value]
/// into a `Result<T, JsonReaderError>` container. This module use _pointer_
/// exclusively as they produce nice error message and goes well with the
/// (convoluted) JSON format of the save games.
mod reader {
    use super::JsonReaderError;
    use serde::de::DeserializeOwned;
    use serde_json::Value;

    // In doc: Clone the JSON value before deserialization
    pub fn pointer_as<T>(json: &Value, pointer: &str) -> Result<T, JsonReaderError>
    where
        T: DeserializeOwned,
    {
        let json = json
            .pointer(pointer)
            .ok_or_else(|| JsonReaderError::InvalidPointer(pointer.to_string()))?;

        Ok(serde_json::from_value(json.clone())?)
    }

    // Very similar to [pointer_as] but simplify type inference a lot at callsite
    // (plus this does not clone the pointed json before returning it)
    pub fn pointer_as_array<'a>(
        json: &'a Value,
        pointer: &'_ str,
    ) -> Result<&'a Vec<Value>, JsonReaderError> {
        let json = json
            .pointer(pointer)
            .ok_or_else(|| JsonReaderError::InvalidPointer(pointer.to_string()))?;

        json.as_array().ok_or_else(|| {
            JsonReaderError::ArrayExpected(pointer.to_string(), json_type(json).to_string())
        })
    }

    // Very similar to [pointer_as] but simplify type inference a lot at callsite
    // (plus this does not clone the pointed json before returning it)
    pub fn pointer_as_object<'a>(
        json: &'a Value,
        pointer: &'_ str,
    ) -> Result<&'a serde_json::map::Map<String, Value>, JsonReaderError> {
        let json = json
            .pointer(pointer)
            .ok_or_else(|| JsonReaderError::InvalidPointer(pointer.to_string()))?;

        json.as_object().ok_or_else(|| {
            JsonReaderError::ObjectExpected(pointer.to_string(), json_type(json).to_string())
        })
    }

    pub(super) fn json_type(value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(..) => "bool",
            Value::Number(..) => "number",
            Value::String(..) => "string",
            Value::Array(..) => "array",
            Value::Object(..) => "object",
        }
    }
}
