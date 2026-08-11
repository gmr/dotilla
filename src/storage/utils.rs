use thiserror::Error;

use super::types::{Database, Namespace, NamespaceName};

/// Returns the namespace with the given name from the database.
pub fn get_namespace(database: &Database, name: &str) -> Result<Namespace, Error> {
    let name = name.to_string();
    match database
        .namespaces
        .lock()
        .unwrap()
        .get(&NamespaceName::try_from(name.to_string()).unwrap())
    {
        Some(namespace) => Ok(namespace.clone()),
        None => Err(Error::NotFound { namespace: name }),
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Namespace not found")]
    NotFound { namespace: String },
}
