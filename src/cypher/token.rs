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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}..{}", self.start, self.end)
    }
}

#[derive(Clone, Debug, PartialEq, strum::AsRefStr)]
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

impl fmt::Display for TokenKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Keyword(value) => value.to_string(),
            Self::Identifier(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::String(value) => value.to_string(),
            Self::Parameter(value) => value.to_string(),
            Self::Op(value) => value.to_string(),
            Self::Punct(value) => value.to_string(),
            Self::Eof => "EOF".to_string(),
        };
        formatter.write_str(value.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, strum::AsRefStr, strum::Display, strum::EnumString)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
pub enum Keyword {
    All,
    And,
    Any,
    As,
    Asc,
    Ascending,
    By,
    Call,
    Case,
    Contains,
    Constraint,
    Count,
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
    Group,
    Groups,
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
    Path,
    Paths,
    Remove,
    Return,
    Set,
    Shortest,
    Single,
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

#[derive(Clone, Debug, PartialEq, strum::AsRefStr, strum::Display, strum::EnumString)]
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

#[derive(Clone, Debug, PartialEq, strum::AsRefStr, strum::EnumString)]
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
    #[strum(serialize = "!")]
    Not,
    #[strum(serialize = "&")]
    And,
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
            Punct::Not => "!",
            Punct::And => "&",
        })
    }
}
