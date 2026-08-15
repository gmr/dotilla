use apache_avro::AvroSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
        database.system.get_item(name).await
    }

    /// Save the config to the database
    pub async fn save(
        &self,
        database: &database::Database,
        name: &str,
    ) -> Result<(), errors::Error> {
        database.system.put_item(name, self).await
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

/// Namespace for a database with handles to the keyspaces to the different keyspace types
pub struct Namespace {
    pub name: types::NamespaceName,
    pub locale: String,
    pub case_insensitive: bool,
    pub collation_strength: CollationStrength,
    pub database: Arc<database::Database>,
    pub system: keyspace::Keyspace,
    pub nodes: keyspace::Keyspace,
    pub edges: keyspace::Keyspace,
    pub labels: keyspace::Keyspace,
    pub vectors: keyspace::Keyspace,
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
                config.save(database, name).await?;
                let keyspaces = keyspace::Keyspaces::open(database, name).await?;
                Ok(Self {
                    name: namespace_name,
                    locale: config.locale,
                    case_insensitive: config.case_insensitive,
                    collation_strength: config.collation_strength,
                    database: Arc::clone(database),
                    system: keyspaces.system,
                    nodes: keyspaces.nodes,
                    edges: keyspaces.edges,
                    labels: keyspaces.labels,
                    vectors: keyspaces.vectors,
                })
            }
            Err(err) => Err(err),
        }
    }

    /// Deletes the namespace and its child keyspaces.
    pub async fn delete(&self) -> Result<(), errors::Error> {
        let keyspaces = keyspace::Keyspaces::open(&self.database, &self.name).await?;
        keyspaces.delete(&self.database).await?;
        let name = self.name.as_ref();
        self.database.system.remove_item(name).await?;
        Ok(())
    }

    pub async fn details(&self) -> Result<Details, errors::Error> {
        let keyspaces = keyspace::Keyspaces::open(&self.database, &self.name).await?;

        let system_future = keyspaces.system.details();
        let nodes_future = keyspaces.nodes.details();
        let edges_future = keyspaces.edges.details();
        let labels_future = keyspaces.labels.details();
        let vectors_future = keyspaces.vectors.details();

        let (system, nodes, edges, labels, vectors) = tokio::try_join!(
            system_future,
            nodes_future,
            edges_future,
            labels_future,
            vectors_future
        )?;

        Ok(Details {
            name: self.name.clone(),
            locale: self.locale.clone(),
            case_insensitive: self.case_insensitive,
            collation_strength: self.collation_strength.clone(),
            size_on_disk: system.size_on_disk
                + nodes.size_on_disk
                + edges.size_on_disk
                + labels.size_on_disk
                + vectors.size_on_disk,
            wasted_space: system.wasted_space
                + nodes.wasted_space
                + edges.wasted_space
                + labels.wasted_space
                + vectors.wasted_space,
            system,
            nodes,
            edges,
            labels,
            vectors,
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
        Ok(Self {
            name: namespace_name,
            locale: config.locale,
            case_insensitive: config.case_insensitive,
            collation_strength: config.collation_strength,
            database: Arc::clone(database),
            system: keyspaces.system,
            nodes: keyspaces.nodes,
            edges: keyspaces.edges,
            labels: keyspaces.labels,
            vectors: keyspaces.vectors,
        })
    }

    /// Returns the next available ID for the given name, incrementing the internal counter.
    pub async fn get_next_id(&self, name: &str) -> Result<u64, errors::Error> {
        let key = format!("{name}_id");
        let id: types::Value = match self.system.get_item(&key).await {
            Ok(types::Value::UInt64(id)) => types::Value::UInt64(id + 1),
            Ok(_) => types::Value::UInt64(0),
            Err(errors::Error::NotFound) => types::Value::UInt64(0),
            Err(err) => return Err(err),
        };
        self.system.put_item(&key, &id).await?;
        match id.as_u64() {
            Some(id) => Ok(id),
            None => Err(errors::Error::ValueError),
        }
    }
}

pub async fn load_all(
    db: &Arc<database::Database>,
) -> Result<HashMap<Name, Namespace>, errors::Error> {
    let mut namespaces = HashMap::new();
    for item in db.system.handle.iter() {
        let name = item.key().unwrap().to_vec();
        let name = String::from_utf8(name).unwrap();
        let ns = Namespace::get(&db.clone(), &name).await?;
        namespaces.insert(ns.name.clone(), ns);
    }
    Ok(namespaces)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helpers::TestContext;
    use test_context::test_context;

    #[test_context(TestContext)]
    #[tokio::test]
    async fn test_create_get_delete_node(ctx: &mut TestContext) {
        let state = ctx.state.clone();
        let ns = Namespace::create(&state.database, "test", None, None, None)
            .await
            .unwrap();
        let id = ns.get_next_id("nodes").await.unwrap();
        assert!(id == 0);
        let id = ns.get_next_id("nodes").await.unwrap();
        assert!(id == 1);
        let id = ns.get_next_id("nodes").await.unwrap();
        assert!(id == 2);
    }
}
