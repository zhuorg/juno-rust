use crate::models::{Number, Value};
use std::{collections::HashMap, convert::From};

impl From<serde_json::Number> for Number {
	fn from(value: serde_json::Number) -> Self {
		if value.is_f64() {
			return Number::Decimal(value.as_f64().unwrap());
		} else if value.is_i64() {
			return Number::SignedInteger(value.as_i64().unwrap().into());
		} else {
			return Number::UnsignedInteger(value.as_u64().unwrap().into());
		}
	}
}

impl Into<serde_json::Number> for Number {
	fn into(self) -> serde_json::Number {
		match self {
			Number::SignedInteger(s) => serde_json::Number::from(s),
			Number::UnsignedInteger(u) => serde_json::Number::from(u),
			Number::Decimal(d) => serde_json::Number::from_f64(d).unwrap(),
		}
	}
}

impl From<serde_json::Value> for Value {
	fn from(v: serde_json::Value) -> Self {
		match v {
			serde_json::Value::Null => Value::Null,
			serde_json::Value::Bool(b) => Value::Bool(b),
			serde_json::Value::Number(n) => Value::Number(n.into()),
			serde_json::Value::String(s) => Value::String(s),
			serde_json::Value::Array(arr) => {
				let mut vec: Vec<Value> = vec![];
				for item in arr.into_iter() {
					vec.push(item.into());
				}
				Value::Array(vec)
			}
			serde_json::Value::Object(map) => {
				let mut hashmap: HashMap<String, Value> = HashMap::new();
				for item in map.into_iter() {
					hashmap.insert(item.0, item.1.into());
				}
				Value::Object(hashmap)
			}
		}
	}
}

impl Into<serde_json::Value> for Value {
	fn into(self) -> serde_json::Value {
		match self {
			Value::Null => serde_json::Value::Null,
			Value::Bool(b) => serde_json::Value::Bool(b),
			Value::Number(n) => serde_json::Value::Number(n.into()),
			Value::String(s) => serde_json::Value::String(s),
			Value::Array(arr) => {
				let mut vec: Vec<serde_json::Value> = vec![];
				for item in arr.into_iter() {
					vec.push(item.into());
				}
				serde_json::Value::Array(vec)
			}
			Value::Object(hashmap) => {
				let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
				for item in hashmap.into_iter() {
					map.insert(item.0, item.1.into());
				}
				serde_json::Value::Object(map)
			}
		}
	}
}
