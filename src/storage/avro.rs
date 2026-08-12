use apache_avro::{
    AvroResult, AvroSchema, reader::datum::GenericDatumReader, writer::datum::GenericDatumWriter,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn decode<T>(bytes: &[u8]) -> AvroResult<T>
where
    T: AvroSchema + DeserializeOwned,
{
    let mut input = bytes;
    let schema = T::get_schema();
    let reader = GenericDatumReader::builder(&schema).build()?;
    reader.read_deser::<T>(&mut input)
}

pub fn encode<T>(value: &T) -> apache_avro::AvroResult<Vec<u8>>
where
    T: AvroSchema + Serialize,
{
    let schema = T::get_schema();

    GenericDatumWriter::builder(&schema)
        .build()?
        .write_ser_to_vec(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::types::{Label, Node, Table, Value};

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
