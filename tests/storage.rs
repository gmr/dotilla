use bytes::Bytes;

use dotilla::storage::*;

#[test]
fn test_round_trip() {
    let mut properties = types::Properties::new();
    properties.insert(
        types::PropertyName::new("array".to_string()).unwrap(),
        types::PropertyValue::Array(vec![
            types::PropertyValue::String("foo".to_string()),
            types::PropertyValue::String("bar".to_string()),
        ]),
    );
    properties.insert(
        types::PropertyName::new("bool_true".to_string()).unwrap(),
        types::PropertyValue::Bool(true),
    );
    properties.insert(
        types::PropertyName::new("bool_false".to_string()).unwrap(),
        types::PropertyValue::Bool(false),
    );
    properties.insert(
        types::PropertyName::new("string".to_string()).unwrap(),
        types::PropertyValue::String("bar".to_string()),
    );
    properties.insert(
        types::PropertyName::new("int8".to_string()).unwrap(),
        types::PropertyValue::Int8(42),
    );
    properties.insert(
        types::PropertyName::new("int16".to_string()).unwrap(),
        types::PropertyValue::Int16(2048),
    );
    properties.insert(
        types::PropertyName::new("int32".to_string()).unwrap(),
        types::PropertyValue::Int16(),
    );

    let encoded: Bytes = encode::table(&properties);
    let decoded: types::Properties = decode::properties(encoded).expect("failed to decode");
    assert_eq!(properties, decoded);
}
