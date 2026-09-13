use thiserror::Error;

use super::token::{Span, TokenKind};

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("encoding error: {0}")]
    EncodingError(#[from] std::string::FromUtf8Error),
    #[error("invalid escape sequence: {span}")]
    InvalidEscape { span: Span },
    #[error("invalid identifier: {span}")]
    InvalidIdentifier { span: Span },
    #[error("invalid parameter: {span}")]
    InvalidParameter { span: Span },
    #[error("not implemented")]
    NotImplemented,
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
    #[error("token parse error")]
    TokenParseError(#[from] strum::ParseError),
    #[error("unexpected character {byte} at {span}")]
    UnexpectedByte { byte: u8, span: Span },
    #[error("unexpected end of tokens")]
    UnexpectedEof,
    #[error("unexpected token: {token:?} in {location}")]
    UnexpectedToken { location: String, token: TokenKind },
    #[error("unterminated comment: {span}")]
    UnterminatedComment { span: Span },
    #[error("unterminated identifier: {span}")]
    UnterminatedIdentifier { span: Span },
    #[error("unterminated string: {span}")]
    UnterminatedString { span: Span },
}

impl Error {
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::EncodingError(_) => None,
            Error::InvalidEscape { span } => Some(*span),
            Error::InvalidIdentifier { span } => Some(*span),
            Error::InvalidParameter { span } => Some(*span),
            Error::NotImplemented => None,
            Error::NumberOutOfRange { span } => Some(*span),
            Error::ParseError { span, .. } => Some(*span),
            Error::ParseFloatError { span, .. } => Some(*span),
            Error::TokenParseError(_) => None,
            Error::UnexpectedByte { span, .. } => Some(*span),
            Error::UnexpectedEof => None,
            Error::UnexpectedToken { .. } => None,
            Error::UnterminatedComment { span } => Some(*span),
            Error::UnterminatedIdentifier { span } => Some(*span),
            Error::UnterminatedString { span } => Some(*span),
        }
    }
}
