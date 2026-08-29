use thiserror::Error;

use super::token::Span;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("encoding error: {0}")]
    EncodingError(#[from] std::string::FromUtf8Error),
    #[error("invalid escape sequence: {span}")]
    InvalidEscape { span: Span },
    #[error("invalid identifier: {span}")]
    InvalidIdentifier { span: Span },
    #[error("invalid parameter: {span}")]
    InvalidParameter { span: Span },
    #[error("number out of range: {span}")]
    NumberOutOfRange { span: Span },
    #[error("parse error `{source}` at {span}")]
    ParseError {
        #[source]
        source: std::num::ParseIntError,
        span: Span,
    },
    #[error("parse float error `{source}` at {span}")]
    ParseFloatError {
        #[source]
        source: std::num::ParseFloatError,
        span: Span,
    },
    #[error("unexpected character {byte} at {span}")]
    UnexpectedByte { byte: u8, span: Span },
    #[error("unterminated comment: {span}")]
    UnterminatedComment { span: Span },
    #[error("unterminated identifier: {span}")]
    UnterminatedIdentifier { span: Span },
    #[error("unterminated string: {span}")]
    UnterminatedString { span: Span },
}
