//! Data model for the save game
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};

use crate::json::{reader, Id, IndexedJson, JsonError, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Party {
    pub characters: Vec<Character>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    pub id: Id,
    pub name: Option<String>,
    pub blueprint: String,
    pub experience: u64,
    pub mythic_experience: Option<u64>,
    pub statistics: Vec<Stat>,
    pub alignment: Alignment,
}

impl Character {
    pub fn find_stat(&self, name: &str) -> Option<&Stat> {
        self.statistics.iter().find(|s| s.tpe == name)
    }

    pub fn name(&self) -> String {
        match &self.name {
            Some(n) => n.clone(),
            None => Character::blueprint_to_name(&self.blueprint)
                .unwrap_or_else(|| self.blueprint.clone()),
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
            "8a6986e17799d7d4b90f0c158b31c5b9" => Some("Smilodon"), // Seelah's pet for sure, maybe smilodon in general ?
            "1cbbbb892f93c3d439f8417ad7cbb6aa" => Some("Sosiel"),
            "f72bb7c48bb3e45458f866045448fb58" => None, // Unknown at the moment, let me progress in game. The Queen maybe ?
            "0d37024170b172346b3769df92a971f5" => Some("Regill"),
            "ae766624c03058440a036de90a7f2009" => Some("Wenduag"),
            "1b893f7cf2b150e4f8bc2b3c389ba71d" => Some("Nenio"),
            "a352873d37ec6c54c9fa8f6da3a6b3e1" => Some("Arueshalae"),
            "32a037e97c3d5c54b85da8f639616c57" => Some("Aivu"),
            _ => {
                info!("Unknown party member found: {}", s);
                None
            }
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
    pub base_value: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alignment {
    /// x axis is lawful/chaotic. Negative is lawful.
    pub x: f32,
    /// y axis is good/evil. Negative is evil.
    pub y: f32,
}

impl<'de> Deserialize<'de> for Alignment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;

        let split: Vec<_> = s.split("|").collect();

        match split[..] {
            [x, y] => {
                let x = x.parse::<f32>().map_err(serde::de::Error::custom)?;
                let y = y.parse::<f32>().map_err(serde::de::Error::custom)?;

                Ok(Alignment { x, y })
            }
            _ => Err(serde::de::Error::custom(format!(
                "invalid format: x|y expected but {} found",
                s
            ))),
        }
    }
}

pub fn read_party(index: &IndexedJson) -> Result<Party, JsonError> {
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
        .collect::<Result<Vec<_>, JsonError>>()?;

    Ok(Party { characters })
}

fn read_character(index: &IndexedJson, json: &Value) -> Result<Character, JsonError> {
    let statistics = reader::pointer_as_object(&json, &"/Descriptor/Stats".into())?
        .iter()
        .filter(|(key, _)| key != &"$id")
        .map(|(key, value)| {
            let path = format!("/Descriptor/Stats/{}", key).into();
            let value = index.dereference(value, &path)?;

            trace!("Looking at {} with value {:?}", path, value);
            let stat = serde_json::from_value(value.clone())?;

            Ok(stat)
        })
        .collect::<Result<Vec<_>, JsonError>>()?;

    let id = reader::pointer_as(&json, &"/$id".into())?;
    let name = (match reader::pointer_as(&json, &"/Descriptor/CustomName".into()) {
        Ok(name) => Ok(Some(name)),
        Err(JsonError::InvalidPointer(_)) => Ok(None),
        Err(err) => Err(err),
    })?;
    let blueprint = reader::pointer_as(&json, &"/Descriptor/Blueprint".into())?;
    let experience = reader::pointer_as(&json, &"/Descriptor/Progression/Experience".into())?;
    let mythic_experience =
        reader::pointer_as(&json, &"/Descriptor/Progression/MythicExperience".into());

    // For now let's go with this solution. In the tutorial section that path doesn't exists
    // (since update 0.8). Let's see how it behave once we have finished act one.
    debug!("Read mythic experience with result {:?}", mythic_experience);
    let mythic_experience = mythic_experience.ok();

    // We use the latest alignment value for display purpose, but we will likely have
    // to find the latest alignment change if we want to be able to modify it.
    // TODO Test out in game what happens if we modify m_Vector only, does the history
    // behaves as a CRDT or is it only for display on the UI ?
    let alignment = reader::pointer_as(&json, &"/Descriptor/Alignment/m_Vector".into())?;

    Ok(Character {
        id,
        name,
        blueprint,
        experience,
        mythic_experience,
        statistics,
        alignment,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub id: Id,
    pub armies: Vec<Army>,
    pub money: u64,
    pub kingdom: Option<Kingdom>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Kingdom {
    pub recruits: RecruitsManager,
    pub resources: KingdomResources,
    pub resources_per_turn: KingdomResources,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Army {
    pub id: Id,
    pub movement_points: f64,
    pub squads: Vec<Squad>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Squad {
    #[serde(alias = "$id")]
    pub id: Id,
    #[serde(alias = "Unit")]
    pub unit: String,
    #[serde(alias = "Count")]
    pub count: u64,
}

impl Squad {
    pub fn id_to_name(s: &str) -> Option<String> {
        let opt = match s {
            "34800b32867af5e4f9cdcb328470be16" => Some("Footmen"),
            "adee0dc84ed3f254fa5da3c90d1e8e3c" => Some("Archers"),
            "ff408898eb9e8694cb849869d1dbfa05" => Some("Clerics"),
            "61e12d43771a40241ac05471a60ddb38" => Some("Hellknights"),
            "81eae30d5d2809b429c0f21d942f7207" => Some("Marksmen"),
            "46c629fc950acbf4fb5535c6d2b8c055" => Some("Spearmen"),
            "45743ca252f07b246a2f5107e5a05fae" => Some("Light Cavalry"),
            "24ad1a10190de5c459a4521996dcec4b" => Some("Shield Bearers"),
            _ => {
                info!("Unknown squad type found: {}", s);
                None
            }
        };

        opt.map(str::to_string)
    }
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
    #[serde(alias = "m_Materials")]
    pub materials: u64,
    #[serde(alias = "m_Favors")]
    pub favors: u64,
}

pub fn read_player(index: &IndexedJson) -> Result<Player, JsonError> {
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
                    let squads = reader::pointer_as(&json, &"/Data/m_Squads".into())?;

                    Ok(Army {
                        id,
                        movement_points,
                        squads,
                    })
                })
                .collect::<Result<Vec<_>, JsonError>>()
        })
        .collect::<Result<Vec<_>, JsonError>>()?
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    let id = reader::pointer_as(&index.json, &"/$id".into())?; // Test that out
    let money = reader::pointer_as(&index.json, &"/Money".into())?;

    // Kingdom is actually optional
    let kingdom = reader::pointer_as_value(&index.json, &"/Kingdom".into())
        .ok()
        .filter(|json| json.is_object())
        .map::<Result<Kingdom, JsonError>, _>(|json| {
            let resources = reader::pointer_as(&json, &"/Resources".into())?;
            let resources_per_turn = reader::pointer_as(&json, &"/ResourcesPerTurn".into())?;
            let recruits = reader::pointer_as(&json, &"/RecruitsManager".into())?;

            Ok(Kingdom {
                resources,
                resources_per_turn,
                recruits,
            })
        })
        .transpose()?;

    Ok(Player {
        id,
        armies,
        money,
        kingdom,
    })
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Header {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "CompatibilityVersion")]
    pub compatibility_version: u64,
}

pub fn read_header(index: &IndexedJson) -> Result<Header, JsonError> {
    Ok(serde_json::from_value(index.json.clone())?)
}
