use super::errors;
use apache_avro::AvroSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::task::spawn_blocking;

use super::{avro, database};

pub struct Names {
    pub system: String,
    pub nodes: String,
    pub edges: String,
    pub labels: String,
    pub vectors: String,
}

impl Names {
    pub fn new(name: &str) -> Self {
        Self {
            system: format!("{}_system", name),
            nodes: format!("{}_nodes", name),
            edges: format!("{}_edges", name),
            labels: format!("{}_labels", name),
            vectors: format!("{}_vectors", name),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Details {
    pub size_on_disk: u64,
    pub item_count: usize,
    pub wasted_space: u64,
}

#[derive(Clone)]
pub struct Keyspace {
    pub name: String,
    pub handle: fjall::Keyspace,
}

impl Keyspace {
    /// Opens a keyspace in the database.
    pub async fn open(db: &database::Database, name: &str) -> Result<Self, errors::Error> {
        let db = db.handle.clone();
        let ns = name.to_string();
        let ks = spawn_blocking(move || db.keyspace(&ns, fjall::KeyspaceCreateOptions::default))
            .await??;
        Ok(Self {
            name: name.to_string(),
            handle: ks,
        })
    }

    /// Deletes the keyspace from the database.
    pub async fn delete(&self, db: &database::Database) -> Result<(), errors::Error> {
        let db = db.handle.clone();
        let handle = self.handle.clone();
        spawn_blocking(move || db.delete_keyspace(handle)).await??;
        Ok(())
    }

    /// Returns the runtime details of the keyspace.
    pub async fn details(&self) -> Result<Details, errors::Error> {
        let item_count_future = self.item_count();
        let size_on_disk_future = self.size_on_disk();
        let wasted_space_future = self.wasted_space();
        let (item_count, size_on_disk, wasted_space) =
            tokio::try_join!(item_count_future, size_on_disk_future, wasted_space_future)?;
        Ok(Details {
            item_count,
            size_on_disk,
            wasted_space,
        })
    }

    /// Returns the number of items in the keyspace.
    pub async fn get_item<T>(&self, key: impl AsRef<[u8]>) -> Result<T, errors::Error>
    where
        T: AvroSchema + DeserializeOwned,
    {
        let handle = self.handle.clone();
        let key = key.as_ref().to_vec();
        match spawn_blocking(move || handle.get(key)).await?? {
            Some(value) => {
                let decoded: T = avro::decode(value.as_slice())?;
                Ok(decoded)
            }
            None => Err(errors::Error::NotFound),
        }
    }

    /// Returns the approximate number of items in the keyspace.
    pub async fn item_count(&self) -> Result<usize, errors::Error> {
        let handle = self.handle.clone();
        Ok(spawn_blocking(move || handle.approximate_len()).await?)
    }

    /// Inserts an item into the keyspace.
    pub async fn put_item<T>(&self, key: impl AsRef<[u8]>, value: &T) -> Result<(), errors::Error>
    where
        T: AvroSchema + Serialize,
    {
        let handle = self.handle.clone();
        let key = key.as_ref().to_vec();
        let encoded = avro::encode::<T>(value)?;
        spawn_blocking(move || handle.insert(key, encoded)).await??;
        Ok(())
    }

    /// Removes an item from the keyspace.
    pub async fn remove_item(&self, key: impl AsRef<[u8]>) -> Result<(), errors::Error> {
        let handle = self.handle.clone();
        let key = key.as_ref().to_vec();
        spawn_blocking(move || handle.remove(key)).await??;
        Ok(())
    }

    /// Returns the approximate size of the keyspace on disk.
    pub async fn size_on_disk(&self) -> Result<u64, errors::Error> {
        let handle = self.handle.clone();
        Ok(spawn_blocking(move || handle.disk_space()).await?)
    }

    /// Returns the approximate amount of wasted space in the keyspace.
    pub async fn wasted_space(&self) -> Result<u64, errors::Error> {
        let handle = self.handle.clone();
        Ok(spawn_blocking(move || handle.fragmented_blob_bytes()).await?)
    }
}

#[derive(Clone)]
pub struct Keyspaces {
    pub system: Keyspace,
    pub nodes: Keyspace,
    pub edges: Keyspace,
    pub labels: Keyspace,
    pub vectors: Keyspace,
}

pub struct KeyspacesDetails {
    pub system: Details,
    pub nodes: Details,
    pub edges: Details,
    pub labels: Details,
    pub vectors: Details,
}

impl Keyspaces {
    /// Opens a new keyspaces instance with the given name.
    pub async fn open(db: &database::Database, name: &str) -> Result<Self, errors::Error> {
        let names = Names::new(name);
        let (system, nodes, edges, labels, vectors) = tokio::try_join!(
            Keyspace::open(db, &names.system),
            Keyspace::open(db, &names.nodes),
            Keyspace::open(db, &names.edges),
            Keyspace::open(db, &names.labels),
            Keyspace::open(db, &names.vectors)
        )?;
        Ok(Self {
            system,
            nodes,
            edges,
            labels,
            vectors,
        })
    }

    /// Deletes all keyspaces.
    pub async fn delete(&self, db: &database::Database) -> Result<(), errors::Error> {
        let _ = tokio::try_join!(
            self.system.delete(db),
            self.nodes.delete(db),
            self.edges.delete(db),
            self.labels.delete(db),
            self.vectors.delete(db)
        )?;
        Ok(())
    }

    /// Returns the details of all keyspaces.
    pub async fn details(&self) -> Result<KeyspacesDetails, errors::Error> {
        let system_future = self.system.details();
        let nodes_future = self.nodes.details();
        let edges_future = self.edges.details();
        let labels_future = self.labels.details();
        let vectors_future = self.vectors.details();
        let (system, nodes, edges, labels, vectors) = tokio::try_join!(
            system_future,
            nodes_future,
            edges_future,
            labels_future,
            vectors_future
        )?;
        Ok(KeyspacesDetails {
            system,
            nodes,
            edges,
            labels,
            vectors,
        })
    }
}
