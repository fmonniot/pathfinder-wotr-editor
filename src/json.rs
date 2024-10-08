//! This module describe our utilities to work with JSON
//! and in particular with the quirks of the Unity data model.

use serde::{Deserialize, Serialize};
pub use serde_json::Value;
use std::collections::BTreeMap;
use std::convert::From;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonPointer(String);

impl From<&str> for JsonPointer {
    fn from(s: &str) -> Self {
        JsonPointer(s.to_string())
    }
}

impl From<String> for JsonPointer {
    fn from(s: String) -> Self {
        JsonPointer(s)
    }
}

impl std::fmt::Display for JsonPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct Id(String);

// We allow dead code because the fields are used via
// the Debug derivation.
#[allow(dead_code)]
#[derive(Debug)]
pub enum JsonError {
    ArrayExpected(JsonPointer, String),  // path and actual type
    ObjectExpected(JsonPointer, String), // path and actual type
    WrongType(String, String),           // when there is no path information ((actual, expected))

    InvalidReference(JsonPointer, Id), // path and $ref value
    InvalidPointer(JsonPointer),
    UnknownId(Id),

    MissingId(JsonPointer), // when there is no $id in the new value (path)
    Deserialization(serde_json::Error),
}

impl From<serde_json::Error> for JsonError {
    fn from(err: serde_json::Error) -> Self {
        JsonError::Deserialization(err)
    }
}

#[derive(Debug, Clone)]
pub struct IndexedJson {
    pub json: Value,
    index: BTreeMap<Id, JsonPointer>,
}

impl IndexedJson {
    pub fn new(json: Value) -> IndexedJson {
        let mut index = BTreeMap::new();
        build_index(&json, "", &mut index);

        IndexedJson { json, index }
    }

    /// Given an id, get the associated pointer for its JSON value
    #[allow(dead_code)]
    pub fn pointer_for(&self, id: Id) -> Result<JsonPointer, JsonError> {
        self.index.get(&id).cloned().ok_or(JsonError::UnknownId(id))
    }

    /// Get the value following a JSON pointer `path`. If the pointed node is a JSON
    /// object containing the field `$ref`, return the JSON node with the associated
    /// `$id`.
    pub fn dereference<'a>(
        &'a self,
        value: &'a Value,
        path: &JsonPointer,
    ) -> Result<&'a Value, JsonError> {
        let sta = value.as_object().ok_or_else(|| {
            JsonError::ObjectExpected(path.clone(), reader::json_type(value).to_string())
        })?;

        match sta
            .get("$ref")
            .and_then(|j| j.as_str())
            .map(|s| Id(s.to_string()))
        {
            Some(reference) => self
                .index
                .get(&reference)
                .and_then(|pointer| self.json.pointer(&pointer.0))
                .ok_or_else(|| JsonError::InvalidReference(path.clone(), reference)),
            None => Ok(value),
        }
    }

    pub fn patch(&mut self, patch: &JsonPatch) -> Result<(), JsonError> {
        match patch {
            JsonPatch::None => Ok(()),
            JsonPatch::Id { id, new_value } => {
                // Clone the pointer to release the immutable reference to self
                let pointer = self
                    .index
                    .get(id)
                    .cloned()
                    .ok_or_else(|| JsonError::UnknownId(id.clone()))?;

                let value = self
                    .json
                    .pointer_mut(&pointer.0)
                    .ok_or_else(|| JsonError::InvalidPointer(pointer.clone()))?;

                if !new_value.contains_key("$id") {
                    return Err(JsonError::MissingId(pointer));
                }

                *value = Value::Object(new_value.clone());

                Ok(())
            }
            JsonPatch::Pointer { pointer, new_value } => {
                let value = self.json.pointer_mut(&pointer.0).unwrap();

                *value = new_value.clone();

                Ok(())
            }
            JsonPatch::IdPointed {
                id,
                pointer,
                new_value,
            } => {
                // Clone the pointer to release the immutable reference to self
                let id_pointer = self
                    .index
                    .get(id)
                    .cloned()
                    .ok_or_else(|| JsonError::UnknownId(id.clone()))?;
                let separator = if pointer.0.starts_with('/') { "" } else { "/" };
                let real_pointer = format!("{}{}{}", id_pointer.0, separator, pointer.0);

                let value = self.json.pointer_mut(&real_pointer).unwrap_or_else(|| {
                    panic!(
                        "IdPointed ({:?}:{:?}, resolved as {}) expected a value but none found",
                        id, pointer, real_pointer
                    )
                });

                *value = new_value.clone();

                Ok(())
            }
        }
    }

    pub fn bytes(&self) -> Result<Vec<u8>, JsonError> {
        Ok(serde_json::to_vec(&self.json)?)
    }
}

#[derive(Debug)]
pub enum JsonPatch {
    None,
    Id {
        id: Id,
        new_value: serde_json::Map<String, Value>,
    },
    Pointer {
        pointer: JsonPointer,
        new_value: Value,
    },
    IdPointed {
        id: Id,
        pointer: JsonPointer, // point relatively to the targeted Id
        new_value: Value,
    },
}

impl JsonPatch {
    // Can only work with pointer, not with $id
    // When working with $id, we need an object (which would include the $id field, although we can it ourselves)
    #[allow(dead_code)]
    pub fn u64(pointer: JsonPointer, value: u64) -> JsonPatch {
        JsonPatch::Pointer {
            pointer,
            new_value: serde_json::to_value(value).unwrap(),
        }
    }

    pub fn string(pointer: JsonPointer, value: String) -> JsonPatch {
        JsonPatch::Pointer {
            pointer,
            new_value: serde_json::to_value(value).unwrap(),
        }
    }

    pub fn id_at_pointer(id: Id, pointer: JsonPointer, new_value: Value) -> JsonPatch {
        JsonPatch::IdPointed {
            id,
            pointer,
            new_value,
        }
    }

    // Not sure if I want to keep the Result here or at the application site
    #[allow(dead_code)]
    pub fn by_id(id: Id, json: Value) -> Result<JsonPatch, JsonError> {
        let mut json = match json {
            Value::Object(map) => Ok(map),
            _ => Err(JsonError::WrongType(
                reader::json_type(&json).to_string(),
                "Object".to_string(),
            )),
        }?;

        if json.get("$id").is_none() {
            json.insert("$id".to_string(), Value::String(id.0.clone()));
        }

        Ok(JsonPatch::Id {
            id,
            new_value: json,
        })
    }
}

fn build_index(json: &Value, path: &str, index: &mut BTreeMap<Id, JsonPointer>) {
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
                .and_then(|id| index.insert(Id(id.to_string()), JsonPointer(path.to_string())));

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
/// into a `Result<T, JsonError>` container. This module use _pointer_
/// exclusively as they produce nice error message and goes well with the
/// (convoluted) JSON format of the save games.
pub mod reader {
    use super::{JsonError, JsonPointer};
    use serde::de::DeserializeOwned;
    use serde_json::Value;

    // In doc: Clone the JSON value before deserialization
    pub fn pointer_as<T>(json: &Value, pointer: &JsonPointer) -> Result<T, JsonError>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_value(pointer_as_value(json, pointer)?)?)
    }

    // In doc: Clone the JSON value before deserialization
    pub fn pointer_as_value(json: &Value, pointer: &JsonPointer) -> Result<Value, JsonError> {
        let json = json
            .pointer(&pointer.0)
            .ok_or_else(|| JsonError::InvalidPointer(pointer.clone()))?;

        Ok(json.clone())
    }

    // Very similar to [pointer_as] but simplify type inference a lot at callsite
    // (plus this does not clone the pointed json before returning it)
    pub fn pointer_as_array<'a>(
        json: &'a Value,
        pointer: &'_ JsonPointer,
    ) -> Result<&'a Vec<Value>, JsonError> {
        let json = json
            .pointer(&pointer.0)
            .ok_or_else(|| JsonError::InvalidPointer(pointer.clone()))?;

        json.as_array()
            .ok_or_else(|| JsonError::ArrayExpected(pointer.clone(), json_type(json).to_string()))
    }

    // Very similar to [pointer_as] but simplify type inference a lot at callsite
    // (plus this does not clone the pointed json before returning it)
    pub fn pointer_as_object<'a>(
        json: &'a Value,
        pointer: &'_ JsonPointer,
    ) -> Result<&'a serde_json::map::Map<String, Value>, JsonError> {
        let json = json
            .pointer(&pointer.0)
            .ok_or_else(|| JsonError::InvalidPointer(pointer.clone()))?;

        json.as_object()
            .ok_or_else(|| JsonError::ObjectExpected(pointer.clone(), json_type(json).to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> (Value, Value) {
        let base: Value = serde_json::from_str(
            r#"
        {
            "$id": "1",
            "other": {
                "$id": "2",
                "value": 42
            }
        }"#,
        )
        .unwrap();
        let expected: Value = serde_json::from_str(
            r#"
        {
            "$id": "1",
            "other": {
                "$id": "2",
                "value": 7
            }
        }"#,
        )
        .unwrap();

        (base, expected)
    }

    #[test]
    fn indexed_json_can_patch_u64_by_pointer() {
        let (base, expected) = fixtures();
        let mut index = IndexedJson::new(base);
        let patch = JsonPatch::u64("/other/value".into(), 7);

        index.patch(&patch).unwrap();

        assert_eq!(index.json, expected);
    }

    #[test]
    fn indexed_json_can_patch_by_id_with_id_in_patch() {
        let (base, expected) = fixtures();
        let mut index = IndexedJson::new(base);
        let patch = JsonPatch::by_id(
            Id("2".to_string()),
            serde_json::from_str(r#"{"$id":"2","value":7}"#).unwrap(),
        )
        .unwrap();

        index.patch(&patch).unwrap();

        assert_eq!(index.json, expected);
    }

    #[test]
    fn indexed_json_can_patch_by_id_without_id_in_patch() {
        let (base, expected) = fixtures();
        let mut index = IndexedJson::new(base);
        let patch = JsonPatch::by_id(
            Id("2".to_string()),
            serde_json::from_str(r#"{"value":7}"#).unwrap(),
        )
        .unwrap();

        index.patch(&patch).unwrap();

        assert_eq!(index.json, expected);
    }
}
