use apache_avro::AvroSchema;
use compio::runtime::spawn_blocking;
use serde::Serialize;

use super::{avro, errors, keyspace};

pub enum Command {
    Insert,
    Remove,
}

pub struct Operation {
    pub command: Command,
    pub keyspace: fjall::Keyspace,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

pub struct Batch {
    database: fjall::Database,
    operations: Vec<Operation>,
}

impl Batch {
    /// Returns a new, empty batch for the given database
    pub fn new(database: &fjall::Database) -> Self {
        Self {
            database: database.clone(),
            operations: Vec::new(),
        }
    }

    /// Returns whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Returns the number of operations in the batch.
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Inserts an item into the keyspace, auto-encoding it with Avro.
    pub fn put_item<T>(
        &mut self,
        keyspace: &keyspace::Keyspace,
        key: impl AsRef<[u8]>,
        value: &T,
    ) -> Result<(), errors::Error>
    where
        T: AvroSchema + Serialize + avro::CachedSchema,
    {
        let encoded = avro::encode::<T>(value)?;
        self.operations.push(Operation {
            command: Command::Insert,
            keyspace: keyspace.handle.clone(),
            key: key.as_ref().to_vec(),
            value: encoded,
        });
        Ok(())
    }

    /// Inserts an item into the keyspace without encoding it with Avro.
    pub fn put_item_raw(
        &mut self,
        keyspace: &keyspace::Keyspace,
        key: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
    ) {
        self.operations.push(Operation {
            command: Command::Insert,
            keyspace: keyspace.handle.clone(),
            key: key.as_ref().to_vec(),
            value: value.as_ref().to_vec(),
        })
    }

    /// Adds a remove operation for the given key to the batch.
    pub fn remove(&mut self, keyspace: &keyspace::Keyspace, key: impl AsRef<[u8]>) {
        self.operations.push(Operation {
            command: Command::Remove,
            keyspace: keyspace.handle.clone(),
            key: key.as_ref().to_vec(),
            value: Vec::new(),
        });
    }

    /// Executes the batch, applying all operations to the database.
    pub async fn execute(self) -> Result<(), errors::Error> {
        let mut batch = self.database.batch();
        for op in &self.operations {
            match op.command {
                Command::Insert => batch.insert(&op.keyspace, &op.key, &op.value),
                Command::Remove => batch.remove(&op.keyspace, &op.key),
            }
        }
        spawn_blocking(move || batch.commit()).await??;
        Ok(())
    }
}
