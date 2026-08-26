use apache_avro::{AvroSchema, Schema, Uuid, schema, serde::AvroSchemaComponent};
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{self, Display};
use std::hash::Hash;
use std::sync::LazyLock;
use thiserror::Error;

use super::avro;

/// Macro for validating a string against a set of rules.
macro_rules! validated_string {
    ($name:ident, $label:literal, |$c:ident, $first:ident| $rule:expr) => {
        validated_string!($name, $label, 63, |$c, $first| $rule);
    };
    ($name:ident, $label:literal, $max:literal, |$c:ident, $first:ident| $rule:expr) => {
        #[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(pub String);

        impl $name {
            pub const MAX_LEN: usize = $max;

            pub fn new(raw: impl Into<String>) -> Result<Self, $crate::storage::types::ValueError> {
                let raw: String = raw.into();
                if raw.is_empty() {
                    return Err($crate::storage::types::ValueError::Empty { kind: $label });
                }
                if raw.len() > Self::MAX_LEN {
                    return Err($crate::storage::types::ValueError::TooLong {
                        kind: $label,
                        len: raw.len(),
                        max: Self::MAX_LEN,
                    });
                }
                let ok = raw.chars().enumerate().all(|(i, $c)| {
                    let $first = i == 0;
                    $rule
                });
                if ok {
                    Ok(Self(raw))
                } else {
                    Err($crate::storage::types::ValueError::Chars {
                        kind: $label,
                        value: raw,
                    })
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_inner(self) -> String {
                self.0
            }

            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl AvroSchemaComponent for $name {
            fn get_schema_in_ctxt(
                _: &mut HashSet<schema::Name>,
                _: schema::NamespaceRef,
            ) -> schema::Schema {
                schema::Schema::String
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $crate::storage::types::ValueError;
            fn try_from(raw: &str) -> Result<Self, Self::Error> {
                Self::new(raw)
            }
        }

        impl TryFrom<String> for $name {
            type Error = $crate::storage::types::ValueError;
            fn try_from(raw: String) -> Result<Self, Self::Error> {
                Self::new(raw)
            }
        }

        impl From<$name> for String {
            fn from(v: $name) -> Self {
                v.0
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = $crate::storage::types::ValueError;
            fn from_str(raw: &str) -> Result<Self, Self::Err> {
                Self::new(raw)
            }
        }

        impl $crate::storage::avro::CachedSchema for $name {
            fn cached_schema() -> &'static apache_avro::Schema {
                static SCHEMA: std::sync::LazyLock<apache_avro::Schema> =
                    std::sync::LazyLock::new(<$name as apache_avro::AvroSchema>::get_schema);

                &SCHEMA
            }
        }
    };
}

validated_string!(EdgeLabel, "EdgeLabel", 16383, |c, first| {
    if first {
        c.is_ascii_uppercase()
    } else {
        c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
    }
});

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct EdgeLabels(pub BTreeSet<EdgeLabel>);

impl EdgeLabels {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn contains(&self, label: &EdgeLabel) -> bool {
        self.0.contains(label)
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a EdgeLabel> + 'a {
        self.0.difference(&other.0)
    }

    pub fn get(&self, label: &EdgeLabel) -> Option<&EdgeLabel> {
        self.0.get(label)
    }

    pub fn insert(&mut self, label: EdgeLabel) {
        self.0.insert(label);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &EdgeLabel> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl AvroSchemaComponent for EdgeLabels {
    fn get_schema_in_ctxt(
        named_schemas: &mut HashSet<schema::Name>,
        enclosing_namespace: schema::NamespaceRef,
    ) -> schema::Schema {
        schema::Schema::array(EdgeLabel::get_schema_in_ctxt(
            named_schemas,
            enclosing_namespace,
        ))
        .build()
    }
}

validated_string!(NodeLabel, "NodeLabel", 16383, |c, first| {
    if first {
        c.is_ascii_uppercase()
    } else {
        c.is_ascii_alphanumeric() || c == '_'
    }
});

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct NodeLabels(pub BTreeSet<NodeLabel>);

impl NodeLabels {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn iter(&self) -> impl Iterator<Item = &NodeLabel> {
        self.0.iter()
    }

    pub fn contains(&self, label: &NodeLabel) -> bool {
        self.0.contains(label)
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a NodeLabel> + 'a {
        self.0.difference(&other.0)
    }

    pub fn extend(&mut self, labels: Self) {
        self.0.extend(labels.0);
    }

    pub fn get(&self, label: &NodeLabel) -> Option<&NodeLabel> {
        self.0.get(label)
    }

    pub fn insert(&mut self, label: NodeLabel) {
        self.0.insert(label);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl AvroSchemaComponent for NodeLabels {
    fn get_schema_in_ctxt(
        named_schemas: &mut HashSet<schema::Name>,
        enclosing_namespace: schema::NamespaceRef,
    ) -> schema::Schema {
        schema::Schema::array(NodeLabel::get_schema_in_ctxt(
            named_schemas,
            enclosing_namespace,
        ))
        .build()
    }
}

validated_string!(NamespaceName, "namespace name", 256, |c, _first| {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
});

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AvroSchema)]
#[avro(namespace = "org.dotilla")]
pub enum Value {
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float(f64),
    None(),
    String(String),
    Timestamp(u64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UUID(Uuid),
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int8(&self) -> Option<i8> {
        match self {
            Value::Int8(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_int16(&self) -> Option<i16> {
        match self {
            Value::Int16(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_int32(&self) -> Option<i32> {
        match self {
            Value::Int32(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_int64(&self) -> Option<i64> {
        match self {
            Value::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_timestamp(&self) -> Option<u64> {
        match self {
            Value::Timestamp(t) => Some(*t),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Value::UInt8(u) => Some(*u),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            Value::UInt16(u) => Some(*u),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::UInt32(u) => Some(*u),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::UInt64(u) => Some(*u),
            _ => None,
        }
    }

    pub fn as_uuid(&self) -> Option<Uuid> {
        match self {
            Value::UUID(u) => Some(*u),
            _ => None,
        }
    }
}

impl avro::CachedSchema for Value {
    fn cached_schema() -> &'static Schema {
        static VALUE_SCHEMA: LazyLock<Schema> = LazyLock::new(Value::get_schema);
        &VALUE_SCHEMA
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Table(HashMap<String, Value>);

impl Table {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn extend(&mut self, other: Table) {
        self.0.extend(other.0);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }

    pub fn insert(&mut self, name: String, value: Value) {
        self.0.insert(name, value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter()
    }
}

impl AvroSchemaComponent for Table {
    fn get_schema_in_ctxt(
        named_schemas: &mut HashSet<schema::Name>,
        enclosing_namespace: schema::NamespaceRef,
    ) -> schema::Schema {
        schema::Schema::map(Value::get_schema_in_ctxt(
            named_schemas,
            enclosing_namespace,
        ))
        .build()
    }
}

impl avro::CachedSchema for Table {
    fn cached_schema() -> &'static Schema {
        static TABLE_SCHEMA: LazyLock<Schema> = LazyLock::new(Table::get_schema);
        &TABLE_SCHEMA
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ValueError {
    Chars {
        kind: &'static str,
        value: String,
    },
    TooLong {
        kind: &'static str,
        len: usize,
        max: usize,
    },
    Empty {
        kind: &'static str,
    },
}

impl Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chars { kind, .. } => {
                let kind = kind.to_case(Case::Title);
                write!(f, "{kind} has unsupported characters")
            }
            Self::TooLong { kind, len, max } => {
                write!(f, "{kind} is {len} bytes, exceeds max of {max}")
            }
            Self::Empty { kind } => {
                let kind = kind.to_case(Case::Title);
                write!(f, "{kind} must not be empty")
            }
        }
    }
}
