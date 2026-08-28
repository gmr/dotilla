use thiserror::Error;

use super::token::Span;

#[derive(Error, Debug)]
pub enum Error {
    #[error("encoding error: {0}")]
    EncodingError(#[from] std::string::FromUtf8Error),
    #[error("invalid escape sequence: {span}")]
    InvalidEscape { span: Span },
    #[error("unexpected character {byte} at {span}")]
    UnexpectedByte { byte: u8, span: Span },
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("unterminated identifier: {span}")]
    UnterminatedIdentifier { span: Span },
    #[error("unterminated string: {span}")]
    UnterminatedString { span: Span },
}
