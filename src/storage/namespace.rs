use std::collections::HashMap;
use thiserror::Error;
use tokio::task::spawn_blocking;

use super::types::*;
use super::{avro, database};

pub async fn create(
    database: &Database,
    name: &str,
    locale: Option<String>,
    case_insensitive: Option<bool>,
    collation_strength: Option<CollationStrength>,
) -> Result<Namespace, Error> {
    // Check if the namespace already exists and add it if it doesn't
    match fetch(&database.db, &database.system, name).await {
        Ok(_) => Err(Error::AlreadyExists {
            namespace: name.to_string(),
        }),
        Err(_) => {
            let keyspaces = open_keyspaces(&database.db, name).await?;

            let config = NamespaceConfig {
                locale: locale.unwrap_or(database.default_locale.to_string()),
                case_insensitive: case_insensitive.unwrap_or(false),
                collation_strength: collation_strength.unwrap_or(CollationStrength::Primary),
            };

            // Add the configuration to the system keyspace
            let serialized_config = serde_json::to_string(&config).unwrap();
            database.system.insert(name, serialized_config).unwrap();

            // Add the namespace to the in-memory namespace cache
            let namespace_name = NamespaceName::try_from(name.to_string()).unwrap();
            let namespace = Namespace {
                name: namespace_name.clone(),
                locale: config.locale,
                case_insensitive: config.case_insensitive,
                collation_strength: config.collation_strength,
                system: keyspaces.system,
                nodes: keyspaces.nodes,
                edges: keyspaces.edges,
                labels: keyspaces.labels,
                vectors: keyspaces.vectors,
            };
            database
                .namespaces
                .lock()
                .unwrap()
                .insert(namespace_name, namespace.clone());
            Ok(namespace)
        }
    }
}

/// Delete a namespace by name
pub async fn delete(database: &Database, name: &str) -> Result<(), Error> {
    // Ensure the namespace exists, otherwise raise an error
    let _namespace = fetch(&database.db, &database.system, name).await?;

    // Fetch the keyspaces to delete
    let keyspaces = open_keyspaces(&database.db, name).await?;

    // Build the futures to delete the keyspaces
    let system_future = super::database::delete_keyspace(&database.db, keyspaces.system);
    let edges_future = super::database::delete_keyspace(&database.db, keyspaces.edges);
    let nodes_future = super::database::delete_keyspace(&database.db, keyspaces.nodes);
    let labels_future = super::database::delete_keyspace(&database.db, keyspaces.labels);
    let vectors_future = super::database::delete_keyspace(&database.db, keyspaces.vectors);

    // Delete the keyspaces
    match tokio::try_join!(
        system_future,
        edges_future,
        nodes_future,
        labels_future,
        vectors_future
    ) {
        Ok(_) => {
            // Delete the namespace from the database namespace hashmap
            database
                .namespaces
                .lock()
                .unwrap()
                .remove(&NamespaceName::try_from(name.to_string()).unwrap())
                .ok_or_else(|| Error::NotFound {
                    namespace: name.to_string(),
                })?;

            // Delete the namespace configuration from the system keyspace
            database.system.remove(name).unwrap();
            Ok(())
        }
        Err(err) => Err(Error::Database(err)),
    }
}

/// Return the details of a namespace by name
pub async fn get(database: &Database, name: &str) -> Result<NamespaceDetails, Error> {
    let namespace = fetch(&database.db, &database.system, name).await?;
    let keyspaces = open_keyspaces(&database.db, name).await?;
    Ok(NamespaceDetails {
        name: name.to_string(),
        locale: namespace.locale.clone(),
        case_insensitive: namespace.case_insensitive,
        collation_strength: namespace.collation_strength,
        system: KeyspaceDetails {
            size_on_disk: keyspaces.system.disk_space(),
            item_count: keyspaces.system.approximate_len(),
            wasted_space: keyspaces.system.fragmented_blob_bytes(),
        },
        nodes: KeyspaceDetails {
            size_on_disk: keyspaces.nodes.disk_space(),
            item_count: keyspaces.nodes.approximate_len(),
            wasted_space: keyspaces.nodes.fragmented_blob_bytes(),
        },
        edges: KeyspaceDetails {
            size_on_disk: keyspaces.edges.disk_space(),
            item_count: keyspaces.edges.approximate_len(),
            wasted_space: keyspaces.edges.fragmented_blob_bytes(),
        },
        labels: KeyspaceDetails {
            size_on_disk: keyspaces.labels.disk_space(),
            item_count: keyspaces.labels.approximate_len(),
            wasted_space: keyspaces.labels.fragmented_blob_bytes(),
        },
        vectors: KeyspaceDetails {
            size_on_disk: keyspaces.vectors.disk_space(),
            item_count: keyspaces.vectors.approximate_len(),
            wasted_space: keyspaces.vectors.fragmented_blob_bytes(),
        },
    })
}

// Get a namespace by name
pub async fn fetch(
    db: &fjall::Database,
    system: &fjall::Keyspace,
    name: &str,
) -> Result<Namespace, Error> {
    let namespace = name.to_string();
    let system = system.clone();
    match spawn_blocking(move || system.get(namespace)).await {
        Ok(Ok(None)) => Err(Error::NotFound {
            namespace: name.to_string(),
        }),
        Ok(Ok(Some(bytes))) => {
            let config: NamespaceConfig = serde_json::from_slice(&bytes).unwrap();
            let keyspaces = open_keyspaces(db, name).await?;
            Ok(Namespace {
                name: NamespaceName::try_from(name.to_string()).unwrap(),
                locale: config.locale,
                case_insensitive: config.case_insensitive,
                collation_strength: config.collation_strength,
                system: keyspaces.system,
                nodes: keyspaces.nodes,
                edges: keyspaces.edges,
                labels: keyspaces.labels,
                vectors: keyspaces.vectors,
            })
        }
        Ok(Err(err)) => Err(Error::Open { err }),
        Err(err) => Err(Error::System { err }),
    }
}

/// Return the namespaces from the system keyspace
pub async fn fetch_all(
    db: &fjall::Database,
    system: &fjall::Keyspace,
) -> Result<HashMap<NamespaceName, Namespace>, Error> {
    let mut namespaces: HashMap<NamespaceName, Namespace> = HashMap::new();
    for guard in system.iter() {
        let key_bytes = guard.key().unwrap();
        let key = std::str::from_utf8(&key_bytes).expect("valid utf8");
        namespaces.insert(
            NamespaceName::try_from(key.to_string()).unwrap(),
            fetch(db, system, key).await.unwrap(),
        );
    }
    Ok(namespaces)
}

/// Return a list of all namespace names
pub fn list(database: &Database) -> Vec<NamespaceName> {
    database
        .namespaces
        .lock()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<NamespaceName>>()
}

/// Get the next ID for a keyspace in a namespace
pub async fn get_next_id(namespace: &super::types::Namespace, keyspace: &str) -> u64 {
    let key = format!("{keyspace}_id");
    let id = match namespace.system.get(&key) {
        Ok(Some(value)) => {
            let decoded: Value = avro::decode(&value).unwrap();
            match decoded.as_u64() {
                Some(id) => id + 1,
                None => 0,
            }
        }
        _ => 0,
    };
    namespace
        .system
        .insert(&key, avro::encode(&Value::UInt64(id)).unwrap())
        .unwrap();
    id
}

// Internal Methods

/// Return struct of the keyspace names for a namespace
pub fn keyspace_names(name: &str) -> KeyspaceNames {
    let namespace = name.to_string();
    KeyspaceNames {
        system: format!("{}_system", namespace),
        nodes: format!("{}_nodes", namespace),
        edges: format!("{}_edges", namespace),
        labels: format!("{}_labels", namespace),
        vectors: format!("{}_vectors", namespace),
    }
}

pub async fn open_keyspaces(db: &fjall::Database, name: &str) -> Result<Keyspaces, Error> {
    let names = keyspace_names(name);
    let system_future = super::database::open_keyspace(db, &names.system);
    let nodes_future = super::database::open_keyspace(db, &names.nodes);
    let edges_future = super::database::open_keyspace(db, &names.edges);
    let labels_future = super::database::open_keyspace(db, &names.labels);
    let vectors_future = super::database::open_keyspace(db, &names.vectors);
    let (system, nodes, edges, labels, vectors) = tokio::try_join!(
        system_future,
        nodes_future,
        edges_future,
        labels_future,
        vectors_future
    )?;
    Ok(Keyspaces {
        system,
        nodes,
        edges,
        labels,
        vectors,
    })
}

/// Errors that can occur when loading the configuration.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Namespace already exists")]
    AlreadyExists { namespace: String },

    #[error("Internal database error")]
    Database(#[from] database::Error),

    #[error("Failed to load namespace config")]
    LoadConfig {
        namespace: String,
        err: serde_json::Error,
    },

    #[error("Namespace not found")]
    NotFound { namespace: String },

    #[error("Error opening namespace")]
    Open { err: fjall::Error },

    #[error("Failed to save namespaces")]
    Save { err: fjall::Error },

    #[error("System task error")]
    System { err: tokio::task::JoinError },
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::storage::database;
    use crate::test_helpers::build_state;

    #[tokio::test]
    async fn test_create_get_delete_node() {
        let state = build_state().await;
        let db = database::initialize(&state.config).await.unwrap();
        create(&db, "test", None, None, None).await.unwrap();
        let ns = fetch(&db.db, &db.system, "test").await.unwrap();
        let id = get_next_id(&ns, "nodes").await;
        assert!(id == 0);
        let id = get_next_id(&ns, "nodes").await;
        assert!(id == 1);
        let id = get_next_id(&ns, "nodes").await;
        assert!(id == 2);
    }
}
