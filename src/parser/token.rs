// Cypher 9 Grammar Tokens
use strum::{Display, EnumString};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Parameter(String),
    Op(Op),
    Punct(Punct),
    Eof,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Keyword {
    All,
    And,
    As,
    Asc,
    Ascending,
    By,
    Call,
    Case,
    Contains,
    Create,
    Cypher,
    Delete,
    Desc,
    Descending,
    Detach,
    Distinct,
    Else,
    End,
    Ends,
    Exists,
    False,
    In,
    Is,
    Limit,
    Mandatory,
    Match,
    Merge,
    Null,
    On,
    Optional,
    Or,
    Order,
    Remove,
    Return,
    Set,
    Skip,
    Starts,
    Then,
    True,
    Union,
    Unwind,
    When,
    Where,
    With,
    Xor,
    Yield,
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Op {
    #[strum(serialize = "=")]
    Eq,
    #[strum(serialize = "<>")]
    Ne,
    #[strum(serialize = "<")]
    Lt,
    #[strum(serialize = "<=")]
    Le,
    #[strum(serialize = ">")]
    Gt,
    #[strum(serialize = ">=")]
    Ge,
    #[strum(serialize = "+")]
    Plus,
    #[strum(serialize = "-")]
    Minus,
    #[strum(serialize = "*")]
    Star,
    #[strum(serialize = "/")]
    Slash,
    #[strum(serialize = "%")]
    Percent,
    #[strum(serialize = "^")]
    Caret,
}

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Punct {
    #[strum(serialize = "(")]
    LParen,
    #[strum(serialize = ")")]
    RParen,
    #[strum(serialize = "{{")]
    LBrace,
    #[strum(serialize = "}}")]
    RBrace,
    #[strum(serialize = "[")]
    LBracket,
    #[strum(serialize = "]")]
    RBracket,
    #[strum(serialize = ",")]
    Comma,
    #[strum(serialize = ":")]
    Colon,
    #[strum(serialize = ".")]
    Dot,
    #[strum(serialize = ";")]
    Semi,
    #[strum(serialize = "|")]
    Pipe,
}
