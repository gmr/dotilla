// Cypher 9 Grammar Tokens
use strum::{Display, EnumString};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl std::fmt::Display for Span {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(_f, "{}..{}", self.start, self.end)
    }
}

#[derive(Debug, Clone, PartialEq, Display)]
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

#[derive(Debug, Clone, PartialEq, EnumString, Display)]
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

#[derive(Debug, Clone, PartialEq, Display, EnumString)]
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

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Punct {
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Colon,    // :
    Dot,      // .
    DotDot,   // ..
    Semi,     // ;
    Pipe,     // |
}

// Implement TryFrom to cleanly map a single byte to your enum
impl TryFrom<u8> for Punct {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            b'(' => Ok(Punct::LParen),
            b')' => Ok(Punct::RParen),
            b'{' => Ok(Punct::LBrace),
            b'}' => Ok(Punct::RBrace),
            b'[' => Ok(Punct::LBracket),
            b']' => Ok(Punct::RBracket),
            b',' => Ok(Punct::Comma),
            b':' => Ok(Punct::Colon),
            b'.' => Ok(Punct::Dot),
            b';' => Ok(Punct::Semi),
            b'|' => Ok(Punct::Pipe),
            _ => Err(()),
        }
    }
}
