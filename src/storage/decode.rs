use bytes::{Buf, Bytes};
use std::str;
use std::time::{Duration, UNIX_EPOCH};
use thiserror::Error;

use super::types::{InvalidName, Properties, PropertyName, PropertyValue};

/// Decode a byte stream into a Properties struct
pub fn properties(value: Bytes) -> Result<Properties, DecodeError> {
    let (decoded, _) = by_type(&value)?;
    match decoded.as_properties() {
        Some(props) => Ok(props.clone()),
        None => Err(DecodeError::ParseFailure),
    }
}

fn by_type(value: &Bytes) -> Result<(PropertyValue, usize), DecodeError> {
    let mut buf: Bytes = value.clone();
    let return_value = match buf.get_u8() {
        b'A' => {
            let mut values = Vec::new();
            let length = buf.get_u64();
            let mut remaining = buf.split_to(length as usize);
            while !remaining.is_empty() {
                let (val, consumed) = by_type(&remaining)?;
                remaining.advance(consumed);
                values.push(val);
            }
            PropertyValue::Array(values)
        }
        b't' => PropertyValue::Bool(buf.get_u8() == 1),
        b'b' => PropertyValue::Int8(buf.get_i8()),
        b'U' => PropertyValue::Int16(buf.get_i16()),
        b'I' => PropertyValue::Int32(buf.get_i32()),
        b'L' => PropertyValue::Int64(buf.get_i64()),
        b'f' => PropertyValue::Float(buf.get_f64()),
        b'V' => PropertyValue::None,
        b's' => {
            let length = buf.get_u8();
            let payload: Bytes = buf.split_to(length as usize);
            match str::from_utf8(&payload) {
                Ok(s) => PropertyValue::String(s.to_string()),
                Err(_) => return Err(DecodeError::UTF8Error),
            }
        }
        b'S' => {
            let length = buf.get_u64();
            let payload: Bytes = buf.split_to(length as usize);
            match str::from_utf8(&payload) {
                Ok(s) => PropertyValue::String(s.to_string()),
                Err(_) => return Err(DecodeError::UTF8Error),
            }
        }
        b'F' => {
            let mut properties = Properties::new();
            let length = buf.get_u64();
            let mut remaining = buf.split_to(length as usize);
            while !remaining.is_empty() {
                let (key, consumed) = by_type(&remaining)?;
                remaining.advance(consumed);
                let (value, consumed) = by_type(&remaining)?;
                remaining.advance(consumed);

                let key = match PropertyName::try_from(key.as_str()) {
                    Ok(k) => k,
                    Err(err) => return Err(DecodeError::InvalidPropertyName { err }),
                };
                properties.insert(key, value);
            }
            PropertyValue::Table(properties)
        }
        b'T' => {
            let duration = buf.get_u64();
            PropertyValue::Timestamp(UNIX_EPOCH + Duration::from_secs(duration))
        }
        b'B' => PropertyValue::UInt8(buf.get_u8()),
        b'u' => PropertyValue::UInt16(buf.get_u16()),
        b'i' => PropertyValue::UInt32(buf.get_u32()),
        b'l' => PropertyValue::UInt64(buf.get_u64()),
        _ => return Err(DecodeError::InvalidDataType),
    };
    Ok((return_value, value.len() - buf.len()))
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid data type")]
    InvalidDataType,

    #[error("invalid property name: {err}")]
    InvalidPropertyName { err: InvalidName },

    #[error("parse failure")]
    ParseFailure,

    #[error("utf8 error")]
    UTF8Error,
}
