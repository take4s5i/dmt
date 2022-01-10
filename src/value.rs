use serde::{de::{ Visitor, MapAccess, SeqAccess, }, Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, error::Error, fmt};

#[macro_export]
macro_rules! vmap {
    {$($key: expr => $val: expr), +} => {
        Value::Map({
            use std::collections::HashMap;
            let mut m: HashMap<String, Value> = HashMap::new();
            $(m.insert(($key).to_owned(), $val);),+
            m
        })
    }
}

#[macro_export]
macro_rules! vlist {
    [$($expr: expr), +] => {
        Value::List(vec![$($expr),+])
    }
}

#[macro_export]
macro_rules! vint {
    ($expr:expr) => {
        Value::Int($expr)
    }
}

#[macro_export]
macro_rules! vfloat {
    ($expr:expr) => {
        Value::Float($expr)
    }
}

#[macro_export]
macro_rules! vbool {
    ($expr:expr) => {
        Value::Bool($expr)
    }
}

#[macro_export]
macro_rules! vunit {
    () => {
        Value::Unit
    }
}

#[macro_export]
macro_rules! vstr {
    ($expr: expr) => {
        Value::String(($expr).to_owned())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Map(HashMap<String, Value>),
    List(Vec<Value>),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Unit => serializer.serialize_unit(),
            Value::Int(x) => serializer.serialize_i64(*x),
            Value::Float(x) => serializer.serialize_f64(*x),
            Value::Bool(x) => serializer.serialize_bool(*x),
            Value::String(x) => serializer.serialize_str(x),
            Value::Map(x) => x.serialize(serializer),
            Value::List(x) => x.serialize(serializer),
        }
    }
}

struct ValueVisitor {}

impl <'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "hoge")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::Unit)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::Int(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::Float(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where E: serde::de::Error, {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where E: serde::de::Error, {
        Ok(Value::String(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de> {
        let mut val: Vec<Value> = Vec::new();

        while let Some(v) = seq.next_element()? {
            val.push(v);
        }

        Ok(Value::List(val))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>
    {
        let mut val: HashMap<String, Value> = HashMap::new();

        while let Some((k, v)) = map.next_entry::<String, Value>()? {
            val.insert(k, v);
        }

        Ok(Value::Map(val))
    }
}

impl <'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor{})
    }
}
