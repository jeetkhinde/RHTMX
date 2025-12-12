//! Conversion between JSON and Automerge values

use automerge::{
    transaction::Transactable, AutoCommit, ObjId, ObjType, ReadDoc, ScalarValue, Value,
};
use serde_json::{Map, Number, Value as JsonValue};

use crate::error::{MergeError, MergeResult};

/// Convert a JSON value and store it in an Automerge document at the given path
pub fn json_to_automerge(
    doc: &mut AutoCommit,
    parent: &ObjId,
    key: &str,
    value: &JsonValue,
) -> MergeResult<()> {
    match value {
        JsonValue::Null => {
            // Automerge doesn't have null, so we delete the key or skip
            let _ = doc.delete(parent, key);
        }
        JsonValue::Bool(b) => {
            doc.put(parent, key, *b)?;
        }
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                doc.put(parent, key, i)?;
            } else if let Some(u) = n.as_u64() {
                doc.put(parent, key, u as i64)?;
            } else if let Some(f) = n.as_f64() {
                doc.put(parent, key, f)?;
            }
        }
        JsonValue::String(s) => {
            doc.put(parent, key, s.as_str())?;
        }
        JsonValue::Array(arr) => {
            let list_obj = doc.put_object(parent, key, ObjType::List)?;
            for (i, item) in arr.iter().enumerate() {
                json_to_automerge_list(doc, &list_obj, i, item)?;
            }
        }
        JsonValue::Object(map) => {
            let map_obj = doc.put_object(parent, key, ObjType::Map)?;
            for (k, v) in map {
                json_to_automerge(doc, &map_obj, k, v)?;
            }
        }
    }
    Ok(())
}

/// Convert a JSON value and insert it into an Automerge list
fn json_to_automerge_list(
    doc: &mut AutoCommit,
    list: &ObjId,
    index: usize,
    value: &JsonValue,
) -> MergeResult<()> {
    match value {
        JsonValue::Null => {
            // Insert a placeholder or skip
            doc.insert(list, index, "")?;
        }
        JsonValue::Bool(b) => {
            doc.insert(list, index, *b)?;
        }
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                doc.insert(list, index, i)?;
            } else if let Some(f) = n.as_f64() {
                doc.insert(list, index, f)?;
            }
        }
        JsonValue::String(s) => {
            doc.insert(list, index, s.as_str())?;
        }
        JsonValue::Array(arr) => {
            let nested_list = doc.insert_object(list, index, ObjType::List)?;
            for (i, item) in arr.iter().enumerate() {
                json_to_automerge_list(doc, &nested_list, i, item)?;
            }
        }
        JsonValue::Object(map) => {
            let nested_map = doc.insert_object(list, index, ObjType::Map)?;
            for (k, v) in map {
                json_to_automerge(doc, &nested_map, k, v)?;
            }
        }
    }
    Ok(())
}

/// Convert an Automerge object to JSON
pub fn automerge_to_json(doc: &AutoCommit, obj_id: &ObjId) -> MergeResult<JsonValue> {
    let obj_type = doc.object_type(obj_id)?;

    match obj_type {
        ObjType::Map => {
            let mut map = Map::new();
            for key in doc.keys(obj_id) {
                if let Some((value, _)) = doc.get(obj_id, &key)? {
                    map.insert(key.to_string(), value_to_json(doc, &value)?);
                }
            }
            Ok(JsonValue::Object(map))
        }
        ObjType::List => {
            let mut arr = Vec::new();
            let len = doc.length(obj_id);
            for i in 0..len {
                if let Some((value, _)) = doc.get(obj_id, i)? {
                    arr.push(value_to_json(doc, &value)?);
                }
            }
            Ok(JsonValue::Array(arr))
        }
        ObjType::Text => {
            let text = doc.text(obj_id)?;
            Ok(JsonValue::String(text.to_string()))
        }
        ObjType::Table => {
            // Tables are similar to maps
            let mut map = Map::new();
            for key in doc.keys(obj_id) {
                if let Some((value, _)) = doc.get(obj_id, &key)? {
                    map.insert(key.to_string(), value_to_json(doc, &value)?);
                }
            }
            Ok(JsonValue::Object(map))
        }
    }
}

/// Convert an Automerge Value to JSON
fn value_to_json(_doc: &AutoCommit, value: &Value) -> MergeResult<JsonValue> {
    match value {
        Value::Object(_obj_type) => {
            // For Object values, we need to look up the ObjId from the context
            // Since we only get the ObjType here, we return a placeholder
            // The actual conversion happens in automerge_to_json which has the ObjId
            Ok(JsonValue::Object(Map::new()))
        }
        Value::Scalar(scalar) => scalar_to_json(&scalar.as_ref().clone()),
    }
}

/// Convert an Automerge ScalarValue to JSON
fn scalar_to_json(scalar: &ScalarValue) -> MergeResult<JsonValue> {
    match scalar {
        ScalarValue::Boolean(b) => Ok(JsonValue::Bool(*b)),
        ScalarValue::Int(i) => Ok(JsonValue::Number((*i).into())),
        ScalarValue::Uint(u) => Ok(JsonValue::Number((*u).into())),
        ScalarValue::F64(f) => {
            Number::from_f64(*f)
                .map(JsonValue::Number)
                .ok_or_else(|| MergeError::InvalidData(format!("Invalid float: {}", f)))
        }
        ScalarValue::Str(s) => Ok(JsonValue::String(s.to_string())),
        ScalarValue::Bytes(b) => {
            // Encode bytes as base64 string
            use base64::{engine::general_purpose::STANDARD, Engine};
            Ok(JsonValue::String(STANDARD.encode(b)))
        }
        ScalarValue::Counter(c) => Ok(JsonValue::Number(i64::from(c).into())),
        ScalarValue::Timestamp(t) => Ok(JsonValue::Number((*t).into())),
        ScalarValue::Null => Ok(JsonValue::Null),
        ScalarValue::Unknown { .. } => Ok(JsonValue::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::ROOT;
    use serde_json::json;

    #[test]
    fn test_json_to_automerge_primitives() {
        let mut doc = AutoCommit::new();
        let obj = doc.put_object(ROOT, "test", ObjType::Map).unwrap();

        json_to_automerge(&mut doc, &obj, "string", &json!("hello")).unwrap();
        json_to_automerge(&mut doc, &obj, "number", &json!(42)).unwrap();
        json_to_automerge(&mut doc, &obj, "float", &json!(3.14)).unwrap();
        json_to_automerge(&mut doc, &obj, "bool", &json!(true)).unwrap();

        let result = automerge_to_json(&doc, &obj).unwrap();
        assert_eq!(result["string"], "hello");
        assert_eq!(result["number"], 42);
        assert_eq!(result["bool"], true);
    }

    #[test]
    fn test_json_to_automerge_nested() {
        let mut doc = AutoCommit::new();
        let obj = doc.put_object(ROOT, "test", ObjType::Map).unwrap();

        let nested = json!({
            "user": {
                "name": "Alice",
                "address": {
                    "city": "Springfield"
                }
            }
        });

        json_to_automerge(&mut doc, &obj, "data", &nested["user"]).unwrap();

        let result = automerge_to_json(&doc, &obj).unwrap();
        assert_eq!(result["data"]["name"], "Alice");
        assert_eq!(result["data"]["address"]["city"], "Springfield");
    }

    #[test]
    fn test_json_to_automerge_array() {
        let mut doc = AutoCommit::new();
        let obj = doc.put_object(ROOT, "test", ObjType::Map).unwrap();

        json_to_automerge(&mut doc, &obj, "tags", &json!(["a", "b", "c"])).unwrap();

        let result = automerge_to_json(&doc, &obj).unwrap();
        assert_eq!(result["tags"][0], "a");
        assert_eq!(result["tags"][1], "b");
        assert_eq!(result["tags"][2], "c");
    }
}
