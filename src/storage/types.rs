use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Mutex;

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
    pub namespaces: Vec<NamespaceDetails>,
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(try_from = "String")]
pub struct NamespaceName(pub String);

impl Display for NamespaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for NamespaceName {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let re = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
        if re.is_match(&raw) {
            Ok(NamespaceName(raw.to_string()))
        } else {
            Err(format!("Database name `{raw}` has unsupported characters."))
        }
    }
}

impl NamespaceName {
    pub fn new(name: &str) -> Self {
        NamespaceName::try_from(name.to_string()).unwrap()
    }
}

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

/// Namespace for a database with handles to the keyspaces to the different keyspace types
pub struct Namespace {
    pub name: NamespaceName,
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
    pub nodes: fjall::Keyspace,
    pub edges: fjall::Keyspace,
    pub vectors: fjall::Keyspace,
}
