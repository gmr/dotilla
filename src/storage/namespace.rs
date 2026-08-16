use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

// Re-export the NamespaceName as Name
pub use super::types::NamespaceName as Name;
use super::{database, errors, keyspace, types};

#[non_exhaustive]
#[repr(u8)]
#[derive(Clone, Debug, Deserialize, Serialize, AvroSchema)]
#[avro(namespace = "org.dotilla")]
pub enum CollationStrength {
    Primary = 0,
    Secondary = 1,
    Tertiary = 2,
    Quaternary = 3,
    Identical = 7,
}

#[derive(Clone, Debug, Deserialize, Serialize, AvroSchema)]
#[avro(namespace = "org.dotilla")]
pub struct Config {
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
}

impl Config {
    /// Load a config from the database
    pub async fn load(database: &database::Database, name: &str) -> Result<Self, errors::Error> {
        database
            .system
            .get_item(format!("ns:{name}").as_str())
            .await
    }

    /// Save the config to the database
    pub async fn save(
        &self,
        database: &database::Database,
        name: &str,
    ) -> Result<(), errors::Error> {
        database
            .system
            .put_item(format!("ns:{name}").as_str(), self)
            .await
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Details {
    pub name: Name,
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
    pub size_on_disk: u64,
    pub wasted_space: u64,
    pub system: keyspace::Details,
    pub nodes: keyspace::Details,
    pub edges: keyspace::Details,
    pub labels: keyspace::Details,
    pub vectors: keyspace::Details,
}

pub struct UniqueIds {
    nodes: AtomicU64,
    edges: AtomicU64,
    labels: AtomicU64,
    vectors: AtomicU64,
}

/// Namespace for a database with handles to the keyspaces to the different keyspace types
pub struct Namespace {
    pub name: types::NamespaceName,
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
    pub database: Arc<database::Database>,
    pub keyspaces: keyspace::Keyspaces,
    pub last_ids: UniqueIds,
}

impl Namespace {
    /// Returns a new Namespace with the given name, locale, case sensitivity, and collation strength.
    pub async fn create(
        database: &Arc<database::Database>,
        name: &str,
        locale: Option<String>,
        case_insensitive: Option<bool>,
        collation_strength: Option<CollationStrength>,
    ) -> Result<Self, errors::Error> {
        let namespace_name = Name::try_from(name.to_string())?;
        let _guard = database.namespace_lock.lock().await;
        match Config::load(database, name).await {
            Ok(_) => Err(errors::Error::NamespaceExists {
                namespace: name.to_string(),
            }),
            Err(errors::Error::NotFound) => {
                let default_locale = database.default_locale.clone();
                let config = Config {
                    locale: locale.unwrap_or(default_locale),
                    case_insensitive: case_insensitive.unwrap_or(false),
                    collation_strength: collation_strength.unwrap_or(CollationStrength::Primary),
                };
                let keyspaces = keyspace::Keyspaces::open(database, name).await?;
                let last_ids = create_last_ids(&keyspaces.system).await?;
                config.save(database, name).await?;
                Ok(Self {
                    name: namespace_name,
                    locale: config.locale,
                    case_insensitive: config.case_insensitive,
                    collation_strength: config.collation_strength,
                    database: Arc::clone(database),
                    keyspaces,
                    last_ids,
                })
            }
            Err(err) => Err(err),
        }
    }

    /// Deletes the namespace and its child keyspaces.
    pub async fn delete(&self) -> Result<(), errors::Error> {
        let _guard = self.database.namespace_lock.lock().await;
        self.keyspaces.delete(&self.database).await?;
        let name = self.name.as_ref();
        self.database
            .system
            .remove_item(format!("ns:{name}"))
            .await?;
        Ok(())
    }

    /// Returns the details of the namespace including disk usage, wasted space, and keyspace details.
    pub async fn details(&self) -> Result<Details, errors::Error> {
        let details = self.keyspaces.details().await?;
        Ok(Details {
            name: self.name.clone(),
            locale: self.locale.clone(),
            case_insensitive: self.case_insensitive,
            collation_strength: self.collation_strength.clone(),
            size_on_disk: details.system.size_on_disk
                + details.nodes.size_on_disk
                + details.edges.size_on_disk
                + details.labels.size_on_disk
                + details.vectors.size_on_disk,
            wasted_space: details.system.wasted_space
                + details.nodes.wasted_space
                + details.edges.wasted_space
                + details.labels.wasted_space
                + details.vectors.wasted_space,
            system: details.system,
            nodes: details.nodes,
            edges: details.edges,
            labels: details.labels,
            vectors: details.vectors,
        })
    }

    /// Returns the namespace with the given name, if it exists.
    pub async fn get(
        database: &Arc<database::Database>,
        name: &str,
    ) -> Result<Self, errors::Error> {
        let namespace_name = Name::try_from(name.to_string())?;
        let config = Config::load(database, name).await?;
        let keyspaces = keyspace::Keyspaces::open(database, name).await?;
        let last_ids = load_last_ids(&keyspaces.system).await?;
        Ok(Self {
            name: namespace_name,
            locale: config.locale,
            case_insensitive: config.case_insensitive,
            collation_strength: config.collation_strength,
            database: Arc::clone(database),
            keyspaces,
            last_ids,
        })
    }

    /// Returns the next available ID for the given name, incrementing the internal counter.
    pub async fn get_next_id(&self, name: &str) -> Result<u64, errors::Error> {
        let (key, id) = match name {
            "nodes" => {
                let value = self
                    .last_ids
                    .nodes
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                ("id:nodes", value)
            }
            "edges" => {
                let value = self
                    .last_ids
                    .edges
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                ("id:edges", value)
            }
            "labels" => {
                let value = self
                    .last_ids
                    .labels
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                ("id:labels", value)
            }
            "vectors" => {
                let value = self
                    .last_ids
                    .vectors
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                ("id:vectors", value)
            }
            _ => {
                return Err(errors::Error::ValueError);
            }
        };
        let value = id + 1;
        self.keyspaces
            .system
            .put_item(&key, &value.to_be_bytes().to_vec())
            .await?;
        Ok(value)
    }
}

/// Creates a new set of last IDs in the system keyspace
async fn create_last_ids(system: &keyspace::Keyspace) -> Result<UniqueIds, errors::Error> {
    let keys = ["id:nodes", "id:edges", "id:labels", "id:vectors"];
    let value: u64 = 0;
    for key in keys {
        system.put_item(key, &value.to_be_bytes().to_vec()).await?;
    }
    Ok(UniqueIds {
        nodes: AtomicU64::new(value),
        edges: AtomicU64::new(value),
        labels: AtomicU64::new(value),
        vectors: AtomicU64::new(value),
    })
}

/// Loads all namespaces from the database off disk
pub async fn load_all(
    db: &Arc<database::Database>,
) -> Result<HashMap<Name, Arc<Namespace>>, errors::Error> {
    let _guard = db.namespace_lock.lock().await;
    let mut namespaces = HashMap::new();
    let items: Vec<fjall::Guard> = db.system.handle.prefix("ns:").collect();
    for item in items {
        let name = item.key()?.to_vec();
        let name = String::from_utf8(name)?;
        let name = name.strip_prefix("ns:").ok_or(errors::Error::ValueError)?;
        let ns = Namespace::get(db, name).await?;
        namespaces.insert(ns.name.clone(), Arc::new(ns));
    }
    Ok(namespaces)
}

/// Loads a unique ID from the system keyspace
async fn load_last_id(system: &keyspace::Keyspace, key: &str) -> Result<u64, errors::Error> {
    let bytes: Vec<u8> = system.get_item(key).await?;
    match bytes[0..8].try_into() {
        Ok(bytes) => Ok(u64::from_be_bytes(bytes)),
        Err(_) => Err(errors::Error::NotFound),
    }
}

/// Loads the last unique IDs from the system keyspace
async fn load_last_ids(system: &keyspace::Keyspace) -> Result<UniqueIds, errors::Error> {
    Ok(UniqueIds {
        nodes: AtomicU64::new(load_last_id(system, "id:nodes").await?),
        edges: AtomicU64::new(load_last_id(system, "id:edges").await?),
        labels: AtomicU64::new(load_last_id(system, "id:labels").await?),
        vectors: AtomicU64::new(load_last_id(system, "id:vectors").await?),
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helpers::TestContext;
    use futures::future::try_join_all;
    use std::collections::HashSet;
    use test_context::test_context;

    #[test_context(TestContext)]
    #[tokio::test]
    async fn test_create_get_delete_node(ctx: &mut TestContext) {
        let state = ctx.state.clone();
        let ns = Namespace::create(&state.database, "test", None, None, None)
            .await
            .unwrap();
        let id = ns.get_next_id("nodes").await.unwrap();
        assert!(id == 1);
        let id = ns.get_next_id("nodes").await.unwrap();
        assert!(id == 2);
        let id = ns.get_next_id("nodes").await.unwrap();
        assert!(id == 3);
    }

    #[test_context(TestContext)]
    #[tokio::test]
    async fn test_get_next_id_concurrency(ctx: &mut TestContext) {
        let ns = Namespace::create(&ctx.state.database, "test", None, None, None)
            .await
            .unwrap();
        let mut futures = vec![];
        for _ in 0..100 {
            futures.push(ns.get_next_id("nodes"));
        }
        let results = try_join_all(futures).await.unwrap();
        assert!(results.len() == 100);
        let unique: HashSet<_> = results.iter().copied().collect();
        assert_eq!(unique.len(), results.len());
    }
}
