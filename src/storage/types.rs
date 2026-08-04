use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(try_from = "String")]
pub struct DatabaseName(String);

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

impl std::fmt::Display for DatabaseName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
