use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
	SignedInteger(i64),
	UnsignedInteger(u64),
	Decimal(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Null,
	Bool(bool),
	Number(Number),
	String(String),
	Array(Vec<Value>),
	Object(HashMap<String, Value>),
}

impl Value {
	pub fn is_null(&self) -> bool {
		if let Value::Null = &self {
			true
		} else {
			false
		}
	}

	pub fn as_null(&self) -> Option<()> {
		if let Value::Null = &self {
			Some(())
		} else {
			None
		}
	}

	pub fn is_bool(&self) -> bool {
		if let Value::Bool(_) = &self {
			true
		} else {
			false
		}
	}

	pub fn as_bool(&self) -> Option<&bool> {
		if let Value::Bool(value) = &self {
			Some(value)
		} else {
			None
		}
	}

	pub fn is_number(&self) -> bool {
		if let Value::Number(_) = &self {
			true
		} else {
			false
		}
	}

	pub fn as_number(&self) -> Option<&Number> {
		if let Value::Number(value) = &self {
			Some(value)
		} else {
			None
		}
	}

	pub fn is_string(&self) -> bool {
		if let Value::String(_) = &self {
			true
		} else {
			false
		}
	}

	pub fn as_string(&self) -> Option<&String> {
		if let Value::String(value) = &self {
			Some(value)
		} else {
			None
		}
	}

	pub fn is_array(&self) -> bool {
		if let Value::Array(_) = &self {
			true
		} else {
			false
		}
	}

	pub fn as_array(&self) -> Option<&Vec<Value>> {
		if let Value::Array(value) = &self {
			Some(value)
		} else {
			None
		}
	}

	pub fn is_object(&self) -> bool {
		if let Value::Object(_) = &self {
			true
		} else {
			false
		}
	}

	pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
		if let Value::Object(value) = &self {
			Some(value)
		} else {
			None
		}
	}
}
