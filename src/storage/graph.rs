use thiserror::Error;

use super::types::{Database, Label, Node, Properties};

pub async fn create_node(
    database: &Database,
    namespace: &str,
    _labels: Vec<Label>,
    properties: &Properties,
) -> Result<(), Error> {
    let _encoded_properties = super::encode::table(properties);
    match super::utils::get_namespace(database, namespace) {
        Ok(_) => {}
        Err(_) => {
            return Err(Error::NamespaceNotFound {
                namespace: namespace.to_string(),
            });
        }
    }
    Ok(())
}

pub async fn get_node() -> Result<Node, Error> {
    Ok(Node {
        id: 0,
        labels: Vec::new(),
        properties: Properties::default(),
    })
}

pub async fn delete_node() {}

pub async fn create_edge() {}

pub async fn delete_edge() {}

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error")]
    DatabaseError(),

    #[error("namespace not found")]
    NamespaceNotFound { namespace: String },
}
