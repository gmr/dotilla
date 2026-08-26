use apache_avro::{
    AvroSchema, Schema, reader::datum::GenericDatumReader, writer::datum::GenericDatumWriter,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::errors;

pub trait CachedSchema {
    fn cached_schema() -> &'static Schema;
}

pub fn decode<T>(bytes: &[u8]) -> Result<T, errors::Error>
where
    T: AvroSchema + DeserializeOwned + CachedSchema,
{
    let mut input = bytes;
    let reader = GenericDatumReader::builder(T::cached_schema()).build()?;
    let value: T = reader.read_deser(&mut input)?;
    Ok(value)
}

pub fn encode<T>(value: &T) -> Result<Vec<u8>, errors::Error>
where
    T: AvroSchema + Serialize + CachedSchema,
{
    let writer = GenericDatumWriter::builder(T::cached_schema()).build()?;
    let bytes = writer.write_ser_to_vec(value)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::graph::Node;
    use crate::storage::types::{NodeLabel, NodeLabels, Table, Value};

    #[test]
    fn test_node_roundtrip() {
        let mut properties = Table::default();
        properties.insert("name".to_string(), Value::String("John".to_string()));
        properties.insert("age".to_string(), Value::UInt8(42));

        let mut labels = NodeLabels::default();
        labels.insert(NodeLabel::new("One").unwrap());
        labels.insert(NodeLabel::new("Two").unwrap());

        let node = Node {
            id: 0,
            labels: labels,
            properties: properties,
        };
        let encoded = encode(&node).unwrap();
        let decoded: Node = decode(&encoded).unwrap();
        assert_eq!(node, decoded);
    }
}
