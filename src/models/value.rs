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
