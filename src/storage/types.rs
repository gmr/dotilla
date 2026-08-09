use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Mutex;

pub struct Registry {
    pub databases: Mutex<HashMap<DatabaseName, Database>>,
}

pub struct Database {
    pub graph: fjall::Database,
    pub vector: lancedb::Connection,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(try_from = "String")]
pub struct DatabaseName(String);

impl Display for DatabaseName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Hash for DatabaseName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl TryFrom<String> for DatabaseName {
    type Error = String;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let re = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
        if re.is_match(&raw) {
            Ok(DatabaseName(raw.to_string()))
        } else {
            Err(format!("Database name `{raw}` has unsupported characters."))
        }
    }
}

impl PartialEq for DatabaseName {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
