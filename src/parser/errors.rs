use thiserror::Error;

use super::token::Span;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected character {byte} at {span}")]
    UnexpectedByte { byte: u8, span: Span },
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("unterminated identifier: {span}")]
    UnterminatedIdentifier { span: Span },
    #[error("unterminated string: {span}")]
    UnterminatedString { span: Span },
}
