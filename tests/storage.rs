use bytes::Bytes;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dotilla::storage::*;

fn time_without_microseconds() -> SystemTime {
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let seconds_only = Duration::from_secs(duration_since_epoch.as_secs());
    UNIX_EPOCH + seconds_only
}

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
        types::PropertyValue::Int32(2147483647),
    );
    properties.insert(
        types::PropertyName::new("int64".to_string()).unwrap(),
        types::PropertyValue::Int64(4147483647),
    );
    properties.insert(
        types::PropertyName::new("negative_int64".to_string()).unwrap(),
        types::PropertyValue::Int64(-4147483647),
    );
    properties.insert(
        types::PropertyName::new("null_value".to_string()).unwrap(),
        types::PropertyValue::None,
    );
    properties.insert(
        types::PropertyName::new("short_string".to_string()).unwrap(),
        types::PropertyValue::String("foo".to_string()),
    );
    let long_string: String = std::iter::repeat_with(|| fastrand::alphanumeric())
        .take(1024)
        .collect();
    properties.insert(
        types::PropertyName::new("long_string".to_string()).unwrap(),
        types::PropertyValue::String(long_string),
    );

    let mut table = types::Properties::new();
    table.insert(
        types::PropertyName::new("short_string".to_string()).unwrap(),
        types::PropertyValue::String("foo".to_string()),
    );

    properties.insert(
        types::PropertyName::new("table".to_string()).unwrap(),
        types::PropertyValue::Table(table),
    );

    properties.insert(
        types::PropertyName::new("timestamp".to_string()).unwrap(),
        types::PropertyValue::Timestamp(time_without_microseconds()),
    );

    properties.insert(
        types::PropertyName::new("uint8".to_string()).unwrap(),
        types::PropertyValue::UInt8(255),
    );
    properties.insert(
        types::PropertyName::new("uint16".to_string()).unwrap(),
        types::PropertyValue::UInt16(65535),
    );
    properties.insert(
        types::PropertyName::new("uint32".to_string()).unwrap(),
        types::PropertyValue::UInt32(4294967295),
    );
    properties.insert(
        types::PropertyName::new("uint64".to_string()).unwrap(),
        types::PropertyValue::UInt64(18446744073709551615),
    );

    let encoded: Bytes = encode::table(&properties);
    let decoded: types::Properties = decode::properties(encoded).expect("failed to decode");
    assert_eq!(properties, decoded);
}
