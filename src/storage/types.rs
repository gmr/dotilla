use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::hash::Hash;
use std::sync::Mutex;
use std::time::SystemTime;

#[non_exhaustive]
#[repr(u8)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CollationStrength {
    Primary = 0,
    Secondary = 1,
    Tertiary = 2,
    Quaternary = 3,
    Identical = 7,
}

/// Top level database handle with the core database and system keyspace, and a map of namespaces.
pub struct Database {
    pub db: fjall::Database,
    pub system: fjall::Keyspace,
    pub default_locale: String,
    pub namespaces: Mutex<HashMap<NamespaceName, Namespace>>,
}

#[derive(serde::Serialize)]
pub struct DatabaseInfo {
    pub size_on_disk: u64,
    pub journal_count: usize,
    pub keyspace_count: usize,
    pub write_buffer_size: u64,
    pub namespaces: Vec<NamespaceDetails>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidName {
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

impl Display for InvalidName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chars { kind, value } => {
                write!(f, "{kind} `{value}` has unsupported characters.")
            }
            Self::TooLong { kind, len, max } => {
                write!(f, "{kind} is {len} bytes, exceeds max of {max}.")
            }
            Self::Empty { kind } => write!(f, "{kind} must not be empty."),
        }
    }
}

impl std::error::Error for InvalidName {}

macro_rules! validated_name {
    ($name:ident, $label:literal, |$c:ident, $first:ident| $rule:expr) => {
        validated_name!($name, $label, 63, |$c, $first| $rule);
    };
    ($name:ident, $label:literal, $max:literal, |$c:ident, $first:ident| $rule:expr) => {
        #[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, serde::Deserialize)]
        #[serde(try_from = "String")]
        pub struct $name(pub String);

        impl $name {
            pub const MAX_LEN: usize = $max;

            pub fn new(
                raw: impl Into<String>,
            ) -> Result<Self, $crate::storage::types::InvalidName> {
                let raw: String = raw.into();
                if raw.is_empty() {
                    return Err($crate::storage::types::InvalidName::Empty { kind: $label });
                }
                if raw.len() > Self::MAX_LEN {
                    return Err($crate::storage::types::InvalidName::TooLong {
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
                    Err($crate::storage::types::InvalidName::Chars {
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
                false
            }
        }

        impl TryFrom<String> for $name {
            type Error = $crate::storage::types::InvalidName;
            fn try_from(raw: String) -> Result<Self, Self::Error> {
                Self::new(raw)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $crate::storage::types::InvalidName;
            fn try_from(raw: &str) -> Result<Self, Self::Error> {
                Self::new(raw)
            }
        }

        impl std::str::FromStr for $name {
            type Err = $crate::storage::types::InvalidName;
            fn from_str(raw: &str) -> Result<Self, Self::Err> {
                Self::new(raw)
            }
        }

        impl From<$name> for String {
            fn from(v: $name) -> Self {
                v.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
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

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.0)
            }
        }
    };
}

pub struct Keyspaces {
    pub nodes: fjall::Keyspace,
    pub edges: fjall::Keyspace,
    pub vectors: fjall::Keyspace,
}

pub struct KeyspaceNames {
    pub nodes: String,
    pub edges: String,
    pub vectors: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyspaceDetails {
    pub size_on_disk: u64,
    pub item_count: usize,
    pub wasted_space: u64,
}

validated_name!(Label, "Label", 16383, |c, first| {
    if first {
        c.is_ascii_alphabetic() || c == '_'
    } else {
        c.is_ascii_alphanumeric() || c == '_'
    }
});

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NamespaceConfig {
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
}

#[derive(Debug, Clone, Serialize)]
pub struct NamespaceDetails {
    pub name: String,
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
    pub nodes: KeyspaceDetails,
    pub edges: KeyspaceDetails,
    pub vectors: KeyspaceDetails,
}

validated_name!(NamespaceName, "namespace name", 256, |c, _first| {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
});

/// Namespace for a database with handles to the keyspaces to the different keyspace types
#[derive(Clone)]
pub struct Namespace {
    pub name: NamespaceName,
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
    pub nodes: fjall::Keyspace,
    pub edges: fjall::Keyspace,
    pub vectors: fjall::Keyspace,
}

validated_name!(PropertyName, "PropertyName", 128, |c, first| {
    if first {
        c.is_ascii_alphabetic() || c == '_'
    } else {
        c.is_ascii_alphanumeric() || c == '_'
    }
});

impl TryFrom<&PropertyValue> for PropertyName {
    type Error = String;
    fn try_from(value: &PropertyValue) -> Result<Self, Self::Error> {
        let s = value.as_str();
        if s.is_empty() {
            return Err("PropertyName cannot be empty".to_string());
        }
        Ok(Self::try_from(s).unwrap())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Array(Vec<PropertyValue>),
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float(f64),
    None,
    String(String),
    Table(Properties),
    Timestamp(SystemTime),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
}

impl PropertyValue {
    pub fn as_str(&self) -> &str {
        match self {
            PropertyValue::String(s) => s.as_str(),
            _ => "",
        }
    }

    pub fn as_properties(&self) -> Option<&Properties> {
        match self {
            PropertyValue::Table(props) => Some(props),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Properties(HashMap<PropertyName, PropertyValue>);

impl Properties {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, name: PropertyName, value: PropertyValue) {
        self.0.insert(name, value);
    }

    pub fn get(&self, name: &PropertyName) -> Option<&PropertyValue> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PropertyName, &PropertyValue)> {
        self.0.iter()
    }
}

impl Default for Properties {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: u64,
    pub labels: Vec<Label>,
    pub properties: Properties,
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub id: u64,
    pub source: u64,
    pub target: u64,
    pub labels: Vec<Label>,
    pub properties: Properties,
}
