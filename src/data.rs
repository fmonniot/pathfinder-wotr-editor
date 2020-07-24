/// Data model for the save game
use serde::{Deserialize, Serialize};

use crate::json::{reader, Id, IndexedJson, JsonReaderError, Value};

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
    pub id: Id,
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
    pub id: Id,
    #[serde(alias = "Type")]
    pub tpe: String,
    #[serde(alias = "m_BaseValue")]
    pub base_value: u64,
}

pub fn read_party(index: &IndexedJson) -> Result<Party, JsonReaderError> {
    let characters = reader::pointer_as_array(&index.json, &"/m_EntityData".into())?
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
    let statistics = reader::pointer_as_object(&json, &"/Descriptor/Stats".into())?
        .iter()
        .filter(|(key, _)| key != &"$id")
        .map(|(key, value)| {
            let value = index.dereference(value, &format!("/Descriptor/Stats/{}", key).into())?;
            let stat = serde_json::from_value(value.clone())?;

            Ok(stat)
        })
        .collect::<Result<Vec<_>, JsonReaderError>>()?;

    let id = reader::pointer_as(&json, &"/$id".into())?;
    let name = reader::pointer_as(&json, &"/Descriptor/CustomName".into())?;
    let blueprint = reader::pointer_as(&json, &"/Descriptor/Blueprint".into())?;
    let experience = reader::pointer_as(&json, &"/Descriptor/Progression/Experience".into())?;
    let mythic_experience =
        reader::pointer_as(&json, &"/Descriptor/Progression/MythicExperience".into())?;

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
    pub id: Id,
    pub armies: Vec<Army>,
    pub money: u64,
    pub recruits: RecruitsManager,
    pub resources: KingdomResources,
    pub resources_per_turn: KingdomResources,
    // TODO modifiers (rankup, claim, etc…)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Army {
    id: Id,
    experience: u64,
    movement_points: f64,
    squads: Vec<Squad>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Squad {
    #[serde(alias = "$id")]
    id: Id,
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
    id: Id,
    #[serde(alias = "Unit")]
    unit: String,
    #[serde(alias = "Count")]
    count: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct KingdomResources {
    #[serde(alias = "$id")]
    pub id: Id,
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
    let armies = reader::pointer_as_array(&index.json, &"/m_GlobalMaps".into())?
        .iter()
        .map(|json| {
            reader::pointer_as_array(&json, &"/m_Armies".into())?
                .iter()
                .filter(|json| {
                    // We only keep the crusaders squads
                    json.pointer("/Data/Faction")
                        .and_then(|v| v.as_str())
                        .filter(|s| s == &"Crusaders")
                        .is_some()
                })
                .map(|json| {
                    let id = reader::pointer_as(&json, &"/$id".into())?;
                    let movement_points = reader::pointer_as(&json, &"/MovementPoints".into())?;
                    let experience = reader::pointer_as(&json, &"/Data/Experience".into())?;
                    let squads = reader::pointer_as(&json, &"/Data/Squads".into())?;

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

    let id = reader::pointer_as(&index.json, &"/$id".into())?; // Test that out
    let resources = reader::pointer_as(&index.json, &"/Kingdom/Resources".into())?;
    let resources_per_turn = reader::pointer_as(&index.json, &"/Kingdom/ResourcesPerTurn".into())?;
    let recruits = reader::pointer_as(&index.json, &"/Kingdom/RecruitsManager".into())?;
    let money = reader::pointer_as(&index.json, &"/Money".into())?;

    Ok(Player {
        id,
        armies,
        money,
        recruits,
        resources,
        resources_per_turn,
    })
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Header {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "CompatibilityVersion")]
    pub compatibility_version: u64,
}

pub fn read_header(index: &IndexedJson) -> Result<Header, JsonReaderError> {
    Ok(serde_json::from_value(index.json.clone())?)
}
