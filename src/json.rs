///! This module describe our utilities to work with JSON
///! and in particular with the quirks of the Unity data model.
pub use serde_json::Value;
use std::collections::BTreeMap;

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

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedJson {
    pub json: Value,
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
    pub fn dereference<'a>(
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

    // While nice to write, this isn't actually useful to us :)
    /*
    fn dereference_mut<'a>(&'a mut self, value: &'a mut Value, path: &str) -> Result<&'a mut Value, JsonReaderError> {
        let sta = match value.as_object_mut() {
            Some(v) => Ok(v),
            None => Err(JsonReaderError::ObjectExpected(path.to_string(), reader::json_type(value).to_string()))
        }?;

        match sta.get_mut("$ref").and_then(|j| j.as_str()) {
            Some(reference) => {
                let pointer = self.index.get(reference).ok_or_else(|| {
                    JsonReaderError::InvalidReference(path.to_string(), reference.to_string())
                })?;

                self.json.pointer_mut(pointer).ok_or_else(|| {
                    JsonReaderError::InvalidReference(path.to_string(), reference.to_string())
                })
            },
            None => Ok(value),
        }
    }
    */

    //* When we start doing actual modification
    fn pointer_mut(&mut self, pointer: &str) -> Option<&mut Value> {
        self.json.pointer_mut(pointer)
    }
    //*/

    // TODO Might need something else that Reader, or rename Reader to a more generic term
    pub fn patch(&mut self, patch: JsonPatch) -> Result<(), JsonReaderError> {

        match patch {
            JsonPatch::Id { id, new_value } => {
                // Clone the pointer to release the immutable reference to self
                let pointer = self.index.get(&id).unwrap().clone(); // TODO Error management

                let value = self.pointer_mut(&pointer).unwrap();

                // TODO Check the $id field is present in the new value
                *value = Value::Object(new_value);

                Ok(())
            }
            JsonPatch::Pointer { pointer, new_value } => {
                let value = self.pointer_mut(&pointer).unwrap();

                *value = new_value;
                
                Ok(())
            }
        }
    }
}

pub enum JsonPatch {
    Id {
        id: String,
        new_value: serde_json::Map<String, Value>
    },
    Pointer {
        pointer: String,
        new_value: Value
    }
}

impl JsonPatch {
    // Can only work with pointer, not with $id
    // When working with $id, we need an object (which would include the $id field, although we can it ourselves)
    pub fn u64(pointer: String, value: u64) -> JsonPatch {
        JsonPatch::Pointer {
            pointer, new_value: serde_json::to_value(value).unwrap()
        }
    }

    pub fn by_id(id: String, json: Value) -> Result<JsonPatch, JsonReaderError> {
        let mut json = match json {
            Value::Object(map) => Ok(map),
            _ => Err(JsonReaderError::ObjectExpected(id.clone(), reader::json_type(&json).to_string()))
        }?;

        if json.get("$id").is_none() {
            json.insert("$id".to_string(), Value::String(id.clone()));
        }

        Ok(JsonPatch::Id {
            id, new_value: json
        })
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

/// A module containing helper functions to read data from a [serde_json::Value]
/// into a `Result<T, JsonReaderError>` container. This module use _pointer_
/// exclusively as they produce nice error message and goes well with the
/// (convoluted) JSON format of the save games.
pub mod reader {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> (Value, Value) {
        let base: Value = serde_json::from_str(r#"
        {
            "$id": "1",
            "other": {
                "$id": "2",
                "value": 42
            }
        }"#).unwrap();
        let expected: Value = serde_json::from_str(r#"
        {
            "$id": "1",
            "other": {
                "$id": "2",
                "value": 7
            }
        }"#).unwrap();

        (base, expected)
    }


    #[test]
    fn indexed_json_can_patch_u64_by_pointer() {
        let (base, expected) = fixtures();
        let mut index = IndexedJson::new(base);
        let patch = JsonPatch::u64("/other/value".to_string(), 7);        

        index.patch(patch).unwrap();

        assert_eq!(index.json, expected);
    }

    #[test]
    fn indexed_json_can_patch_by_id_with_id_in_patch() {
        let (base, expected) = fixtures();
        let mut index = IndexedJson::new(base);
        let patch = JsonPatch::by_id("2".to_string(), serde_json::from_str(r#"{"$id":"2","value":7}"#).unwrap()).unwrap();        

        index.patch(patch).unwrap();

        assert_eq!(index.json, expected);
    }

    #[test]
    fn indexed_json_can_patch_by_id_without_id_in_patch() {
        let (base, expected) = fixtures();
        let mut index = IndexedJson::new(base);
        let patch = JsonPatch::by_id("2".to_string(), serde_json::from_str(r#"{"value":7}"#).unwrap()).unwrap();        

        index.patch(patch).unwrap();

        assert_eq!(index.json, expected);
    }
}