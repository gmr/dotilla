// OpenCypher 9 Tokens
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn display(&self) -> String {
        format!(
            "<{kind}> start: {start}, end: {end}",
            kind = self.kind,
            start = self.span.start,
            end = self.span.end
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}..{}", self.start, self.end)
    }
}

#[derive(Clone, Debug, PartialEq, strum::Display)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(String),
    Integer(u64),
    Float(f64),
    String(String),
    Parameter(String),
    Op(Op),
    Punct(Punct),
    Eof,
}

#[derive(Clone, Debug, PartialEq, strum::Display, strum::EnumString)]
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
    Constraint,
    Create,
    Cypher,
    Delete,
    Desc,
    Descending,
    Detach,
    Distinct,
    Drop,
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
    None,
    Not,
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
    Unique,
    Unwind,
    When,
    Where,
    With,
    Xor,
    Yield,
}

#[derive(Clone, Debug, PartialEq, strum::Display, strum::EnumString)]
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
    #[strum(serialize = "+=")]
    PlusEq,
    #[strum(serialize = "=~")]
    EqTilde,
}

#[derive(Clone, Debug, PartialEq, strum::EnumString)]
pub enum Punct {
    #[strum(serialize = "(")]
    LParen,
    #[strum(serialize = ")")]
    RParen,
    #[strum(serialize = "{")]
    LBrace,
    #[strum(serialize = "}")]
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
    #[strum(serialize = "..")]
    DotDot,
    #[strum(serialize = ";")]
    Semi,
    #[strum(serialize = "|")]
    Pipe,
}

impl fmt::Display for Punct {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Punct::LParen => "(",
            Punct::RParen => ")",
            Punct::LBrace => "{",
            Punct::RBrace => "}",
            Punct::LBracket => "[",
            Punct::RBracket => "]",
            Punct::Comma => ",",
            Punct::Colon => ":",
            Punct::Dot => ".",
            Punct::DotDot => "..",
            Punct::Semi => ";",
            Punct::Pipe => "|",
        })
    }
}
