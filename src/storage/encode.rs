// Encoding inspired by AMQP Table encoding for property values
use bytes::{BufMut, Bytes, BytesMut};

use super::types::{Properties, PropertyValue};

pub fn array(values: &Vec<PropertyValue>) -> Bytes {
    let total_size: usize = values.iter().map(size).sum();
    let mut buf = BytesMut::with_capacity(total_size + 9);
    buf.put_u8(b'A');
    buf.put_u64(total_size as u64);
    for value in values {
        buf.extend_from_slice(encode(value).as_ref());
    }
    buf.freeze()
}

pub fn table(properties: &Properties) -> Bytes {
    let total_size: usize = properties
        .iter()
        .map(|(k, v)| size(&PropertyValue::String(k.to_string())) + size(v))
        .sum();
    let mut buf = BytesMut::with_capacity(total_size + 9);
    buf.put_u8(b'F');
    buf.put_u64(total_size as u64);
    for (key, value) in properties.iter() {
        buf.extend_from_slice(encode(&PropertyValue::String(key.to_string())).as_ref());
        buf.extend_from_slice(encode(value).as_ref());
    }
    buf.freeze()
}

fn encode(value: &PropertyValue) -> Bytes {
    let mut buf = BytesMut::with_capacity(size(value));
    match value {
        PropertyValue::Array(values) => buf.extend_from_slice(array(values).as_ref()),
        PropertyValue::Bool(v) => {
            buf.put_u8(b't');
            buf.put_u8(if *v { 1 } else { 0 });
        }
        PropertyValue::Int8(v) => {
            buf.put_u8(b'b');
            buf.put_i8(*v);
        }
        PropertyValue::Int16(v) => {
            buf.put_u8(b'U');
            buf.put_i16(*v);
        }
        PropertyValue::Int32(v) => {
            buf.put_u8(b'I');
            buf.put_i32(*v);
        }
        PropertyValue::Int64(v) => {
            buf.put_u8(b'L');
            buf.put_i64(*v);
        }
        PropertyValue::Float(v) => {
            buf.put_u8(b'f');
            buf.put_f64(*v);
        }
        PropertyValue::None => buf.put_u8(b'V'),
        PropertyValue::String(v) => {
            if v.len() < 256 {
                buf.put_u8(b's');
                buf.put_u8(v.len() as u8);
            } else {
                buf.put_u8(b'S');
                buf.put_u64(v.len() as u64)
            }
            buf.extend_from_slice(v.as_bytes());
        }
        PropertyValue::Table(t) => buf.extend_from_slice(table(t).as_ref()),
        PropertyValue::Timestamp(v) => {
            let duration = match v.duration_since(std::time::UNIX_EPOCH) {
                Ok(d) => d.as_secs(),
                Err(_) => 0, // @TODO: This is wrong but we need to refactor to raise errors
            };
            buf.put_u8(b'T');
            buf.put_u64(duration);
        }
        // @TODO: Unsigned ints should raise when they are negative
        PropertyValue::UInt8(v) => {
            buf.put_u8(b'B');
            buf.put_u8(*v);
        }
        PropertyValue::UInt16(v) => {
            buf.put_u8(b'u');
            buf.put_u16(*v);
        }
        PropertyValue::UInt32(v) => {
            buf.put_u8(b'i');
            buf.put_u32(*v);
        }
        PropertyValue::UInt64(v) => {
            buf.put_u8(b'l');
            buf.put_u64(*v);
        }
    };
    buf.freeze()
}

fn size(value: &PropertyValue) -> usize {
    match value {
        PropertyValue::Array(values) => {
            let mut total = 9;
            for value in values {
                total += size(value);
            }
            total
        }
        PropertyValue::Bool(_) => 2,
        PropertyValue::Int8(_) => 2,
        PropertyValue::Int16(_) => 3,
        PropertyValue::Int32(_) => 5,
        PropertyValue::Int64(_) => 9,
        PropertyValue::Float(_) => 9,
        PropertyValue::None => 1,
        PropertyValue::String(s) => {
            let n = s.len();
            n + if n > 255 { 9 } else { 2 }
        }
        PropertyValue::Table(table) => {
            let mut total = 9;
            for (key, value) in table.iter() {
                total += size(&PropertyValue::String(key.to_string()));
                total += size(value);
            }
            total
        }
        PropertyValue::Timestamp(_) => 9,
        PropertyValue::UInt8(_) => 2,
        PropertyValue::UInt16(_) => 3,
        PropertyValue::UInt32(_) => 5,
        PropertyValue::UInt64(_) => 9,
    }
}
