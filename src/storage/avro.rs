use apache_avro::{
    AvroSchema, reader::datum::GenericDatumReader, writer::datum::GenericDatumWriter,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::errors;

pub fn decode<T>(bytes: &[u8]) -> Result<T, errors::Error>
where
    T: AvroSchema + DeserializeOwned,
{
    let mut input = bytes;
    let schema = T::get_schema();
    match GenericDatumReader::builder(&schema).build() {
        Ok(reader) => match reader.read_deser::<T>(&mut input) {
            Ok(value) => Ok(value),
            Err(err) => Err(errors::Error::Decoding { err }),
        },
        Err(err) => Err(errors::Error::Decoding { err }),
    }
}

pub fn encode<T>(value: &T) -> Result<Vec<u8>, errors::Error>
where
    T: AvroSchema + Serialize,
{
    let schema = T::get_schema();
    match GenericDatumWriter::builder(&schema).build() {
        Ok(writer) => match writer.write_ser_to_vec(value) {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(errors::Error::Encoding { err }),
        },
        Err(err) => Err(errors::Error::Encoding { err }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::graph::Node;
    use crate::storage::types::{Label, Table, Value};

    #[test]
    fn test_node_roundtrip() {
        let mut properties = Table::default();
        properties.insert("name".to_string(), Value::String("John".to_string()));
        properties.insert("age".to_string(), Value::UInt8(42));
        let node = Node {
            id: 0,
            labels: vec![Label::new("One").unwrap(), Label::new("Two").unwrap()],
            properties: properties,
        };
        let encoded = encode(&node).unwrap();
        let decoded: Node = decode(&encoded).unwrap();
        assert_eq!(node, decoded);
    }
}
