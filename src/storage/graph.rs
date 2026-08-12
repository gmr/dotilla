use thiserror::Error;

use super::avro;
use super::types::{Database, Label, Node, Table};

pub async fn create_node(
    database: &Database,
    namespace: &str,
    labels: Vec<Label>,
    properties: &Table,
) -> Result<(), Error> {
    // Get the next available node ID

    let node = Node {
        id: 0,
        labels,
        properties: properties.clone(),
    };

    let encoded = avro::encode(&node).unwrap();
    let _decoded: Node = avro::decode(&encoded).unwrap();

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
        properties: Table::default(),
    })
}

pub async fn delete_node() {}

pub async fn update_node() {}

pub async fn upsert_node() {}

pub async fn list_nodes() {}

pub async fn search_nodes() {}

pub async fn create_edge() {}

pub async fn delete_edge() {}

pub async fn get_edge() {}

pub async fn update_edge() {}

pub async fn upsert_edge() {}

pub async fn list_edges() {}

pub async fn search_edges() {}

pub async fn get_neighbors() {}

pub async fn traverse() {}

pub async fn find_paths() {}

pub async fn find_shortest_path() {}

pub async fn match_subgraph() {}

pub async fn check_node_exists() {}

pub async fn check_edge_exists() {}

pub async fn bulk_create_nodes() {}

pub async fn bulk_delete_nodes() {}

pub async fn bulk_update_nodes() {}

pub async fn bulk_create_edges() {}

pub async fn bulk_delete_edges() {}

pub async fn bulk_update_edges() {}

pub async fn create_index() {}

pub async fn drop_index() {}

pub async fn start_transaction() {}

pub async fn commit_transaction() {}

pub async fn rollback_transaction() {}

pub async fn get_graph_schema() {}

pub async fn get_graph_stats() {}

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error")]
    DatabaseError(),

    #[error("namespace not found")]
    NamespaceNotFound { namespace: String },
}
