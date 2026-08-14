use apache_avro::AvroSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;
use tokio::task::spawn_blocking;

use super::types::{Database, Label, Node, Table};
use super::{avro, namespace};

pub async fn create_node(
    database: &Database,
    namespace: &str,
    labels: &[Label],
    properties: &Table,
) -> Result<Node, Error> {
    let namespace_name = namespace.to_string();
    match super::utils::get_namespace(database, &namespace_name) {
        Ok(ns) => {
            let id = namespace::get_next_id(&ns, "nodes").await;
            let node = Node {
                id,
                labels: labels.to_vec(),
                properties: properties.clone(),
            };
            put(&ns.nodes, &format!("{id}"), &node).await?;
            Ok(node)
        }
        Err(_) => Err(Error::NamespaceNotFound {
            namespace: namespace.to_string(),
        }),
    }
}

pub async fn get_node(database: &Database, namespace: &str, id: u64) -> Result<Node, Error> {
    let namespace_name = namespace.to_string();
    match super::utils::get_namespace(database, &namespace_name) {
        Ok(ns) => {
            let node: Node = get(&ns.nodes, &format!("{id}")).await?;
            Ok(node)
        }
        Err(_) => Err(Error::NamespaceNotFound {
            namespace: namespace.to_string(),
        }),
    }
}

pub async fn delete_node(database: &Database, namespace: &str, id: u64) -> Result<(), Error> {
    let namespace_name = namespace.to_string();
    match super::utils::get_namespace(database, &namespace_name) {
        Ok(ns) => {
            let key = format!("{id}");
            delete(&ns.nodes, &key).await
        }
        Err(_) => Err(Error::NamespaceNotFound {
            namespace: namespace.to_string(),
        }),
    }
}

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

async fn delete(keyspace: &fjall::Keyspace, key: &str) -> Result<(), Error> {
    let keyspace = keyspace.clone();
    let key = key.to_string();

    match spawn_blocking(move || keyspace.remove(&key)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(Error::Database(err)),
        Err(err) => Err(Error::IO(err)),
    }
}

async fn get<T>(keyspace: &fjall::Keyspace, key: &str) -> Result<T, Error>
where
    T: AvroSchema + DeserializeOwned,
{
    let keyspace = keyspace.clone();
    let key = key.to_string();
    match spawn_blocking(move || keyspace.get(&key)).await {
        Ok(Ok(Some(value))) => {
            let decoded: T = avro::decode(&value).map_err(|e| Error::Corruption { err: e })?;
            Ok(decoded)
        }
        Ok(Ok(None)) => Err(Error::NotFound),
        Ok(Err(err)) => Err(Error::Database(err)),
        Err(err) => Err(Error::IO(err)),
    }
}

async fn put<T>(keyspace: &fjall::Keyspace, key: &str, value: &T) -> Result<(), Error>
where
    T: AvroSchema + Serialize,
{
    let keyspace = keyspace.clone();
    let key = key.to_string();
    let encoded = avro::encode::<T>(value).unwrap();

    match spawn_blocking(move || keyspace.insert(&key, encoded)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(Error::Database(err)),
        Err(err) => Err(Error::IO(err)),
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("corrupted tuple")]
    Corruption { err: apache_avro::Error },

    #[error("database error")]
    Database(#[from] fjall::Error),

    #[error("Failed to execute blocking operation")]
    IO(#[from] tokio::task::JoinError),

    #[error("namespace not found")]
    NamespaceNotFound { namespace: String },

    #[error("not found")]
    NotFound,
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::storage::{database, namespace, types};
    use crate::test_helpers::build_state;

    #[tokio::test]
    async fn test_create_get_delete_node() {
        let state = build_state().await;
        let db = database::initialize(&state.config).await.unwrap();
        let namespace = namespace::create(&db, "test", None, None, None)
            .await
            .unwrap();
        assert!(namespace.name.to_string() == "test".to_string());

        let labels = vec![types::Label("Foo".to_string())];
        let mut properties = types::Table::default();
        properties.insert("foo".to_string(), types::Value::String("bar".to_string()));

        let node = create_node(&db, "test", &labels, &properties)
            .await
            .unwrap();

        assert!(node.labels.len() == 1);
        assert!(node.labels[0].0 == "Foo");
        assert!(
            node.properties.get(&"foo".to_string())
                == Some(&types::Value::String("bar".to_string()))
        );

        let fetched: Node = get_node(&db, "test", node.id).await.unwrap();

        assert!(fetched.id == node.id);
        assert!(fetched.labels.len() == 1);
        assert!(fetched.labels[0].0 == "Foo");
        assert!(
            fetched.properties.get(&"foo".to_string())
                == Some(&types::Value::String("bar".to_string()))
        );

        delete_node(&db, "test", node.id).await.unwrap();
        let fetched = get_node(&db, "test", node.id).await;
        assert!(fetched.is_err());
    }
}
