use std::collections::HashMap;

use super::{errors, token};

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub clauses: Vec<Clause>,
    pub return_clause: Option<Return>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Clause {
    Create,
    Delete,
    Match(Match),
    Merge,
    Set,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub optional: bool,
    pub paths: Vec<Path>,
    pub where_clause: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path(Vec<Segment>);

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Node(Node),
    Edge(Edge),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub variable: Option<Variable>,
    pub labels: Vec<Label>,
    pub properties: Option<HashMap<Property, Literal>>,
    pub predicates: Vec<Predicate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    pub properties: Option<HashMap<Property, Literal>>,
    pub where_clause: Option<Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathLength {
    Range {
        lower: Option<u64>,
        upper: Option<u64>,
    },
    Fixed(u64),
    Any,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeLabels {
    pub variable: Option<Variable>,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub variable: Option<Variable>,
    pub labels: Vec<Label>,
    pub length: Option<PathLength>,
    pub predicate: Option<Predicate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// `<-[...]-`
    Left,
    /// `-[...]->`
    Right,
    /// `<-[...]->`
    Either,
    /// `-[...]-`
    Undirected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Return {
    pub items: Vec<PropertyReference>,
    pub order_by: Vec<OrderBy>,
    pub skip: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    pub item: PropertyReference,
    pub direction: OrderDirection,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::Display, strum::EnumString,
)]
pub enum OrderDirection {
    #[strum(serialize = "ASC")]
    Asc,
    #[strum(serialize = "DESC")]
    Desc,
}

/// Macro for defining string value types (e.g. `Label`, `Variable`, `Property`)
macro_rules! string_value {
    ($name:ident, $label:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_inner(self) -> String {
                self.0
            }

            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

string_value!(Label, "Label");
string_value!(Variable, "Variable");
string_value!(Property, "Property");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    All,
    AllShortestPaths,
    And,
    Any,
    As,
    Asc,
    Ascending,
    By,
    Call,
    Case,
    Contains,
    Count,
    Create,
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
    Group,
    Groups,
    In,
    Inf,
    Infinity,
    Is,
    Limit,
    Match,
    Merge,
    Nan,
    None,
    Not,
    Null,
    Offset,
    On,
    Optional,
    Or,
    Order,
    Path,
    Paths,
    Reduce,
    Remove,
    Return,
    Set,
    Shortest,
    ShortestPath,
    Single,
    Skip,
    Starts,
    Then,
    Trim,
    True,
    Union,
    Unwind,
    When,
    Where,
    With,
    Xor,
    Yield,
}

impl Keyword {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(word: &str) -> Option<Self> {
        let upper = word.to_ascii_uppercase();
        let keyword = match upper.as_str() {
            "ALL" => Keyword::All,
            "ALLSHORTESTPATHS" => Keyword::AllShortestPaths,
            "AND" => Keyword::And,
            "ANY" => Keyword::Any,
            "AS" => Keyword::As,
            "ASC" => Keyword::Asc,
            "ASCENDING" => Keyword::Ascending,
            "BY" => Keyword::By,
            "CALL" => Keyword::Call,
            "CASE" => Keyword::Case,
            "CONTAINS" => Keyword::Contains,
            "COUNT" => Keyword::Count,
            "CREATE" => Keyword::Create,
            "DELETE" => Keyword::Delete,
            "DESC" => Keyword::Desc,
            "DESCENDING" => Keyword::Descending,
            "DETACH" => Keyword::Detach,
            "DISTINCT" => Keyword::Distinct,
            "ELSE" => Keyword::Else,
            "END" => Keyword::End,
            "ENDS" => Keyword::Ends,
            "EXISTS" => Keyword::Exists,
            "FALSE" => Keyword::False,
            "GROUP" => Keyword::Group,
            "GROUPS" => Keyword::Groups,
            "IN" => Keyword::In,
            "INF" => Keyword::Inf,
            "INFINITY" => Keyword::Infinity,
            "IS" => Keyword::Is,
            "LIMIT" => Keyword::Limit,
            "MATCH" => Keyword::Match,
            "MERGE" => Keyword::Merge,
            "NAN" => Keyword::Nan,
            "NONE" => Keyword::None,
            "NOT" => Keyword::Not,
            "NULL" => Keyword::Null,
            "OFFSET" => Keyword::Offset,
            "ON" => Keyword::On,
            "OPTIONAL" => Keyword::Optional,
            "OR" => Keyword::Or,
            "ORDER" => Keyword::Order,
            "PATH" => Keyword::Path,
            "PATHS" => Keyword::Paths,
            "REDUCE" => Keyword::Reduce,
            "REMOVE" => Keyword::Remove,
            "RETURN" => Keyword::Return,
            "SET" => Keyword::Set,
            "SHORTEST" => Keyword::Shortest,
            "SHORTESTPATH" => Keyword::ShortestPath,
            "SINGLE" => Keyword::Single,
            "SKIP" => Keyword::Skip,
            "STARTS" => Keyword::Starts,
            "THEN" => Keyword::Then,
            "TRIM" => Keyword::Trim,
            "TRUE" => Keyword::True,
            "UNION" => Keyword::Union,
            "UNWIND" => Keyword::Unwind,
            "WHEN" => Keyword::When,
            "WHERE" => Keyword::Where,
            "WITH" => Keyword::With,
            "XOR" => Keyword::Xor,
            "YIELD" => Keyword::Yield,
            _ => return None,
        };
        Some(keyword)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::All => "ALL",
            Keyword::AllShortestPaths => "ALLSHORTESTPATHS",
            Keyword::And => "AND",
            Keyword::Any => "ANY",
            Keyword::As => "AS",
            Keyword::Asc => "ASC",
            Keyword::Ascending => "ASCENDING",
            Keyword::By => "BY",
            Keyword::Call => "CALL",
            Keyword::Case => "CASE",
            Keyword::Contains => "CONTAINS",
            Keyword::Count => "COUNT",
            Keyword::Create => "CREATE",
            Keyword::Delete => "DELETE",
            Keyword::Desc => "DESC",
            Keyword::Descending => "DESCENDING",
            Keyword::Detach => "DETACH",
            Keyword::Distinct => "DISTINCT",
            Keyword::Else => "ELSE",
            Keyword::End => "END",
            Keyword::Ends => "ENDS",
            Keyword::Exists => "EXISTS",
            Keyword::False => "FALSE",
            Keyword::Group => "GROUP",
            Keyword::Groups => "GROUPS",
            Keyword::In => "IN",
            Keyword::Inf => "INF",
            Keyword::Infinity => "INFINITY",
            Keyword::Is => "IS",
            Keyword::Limit => "LIMIT",
            Keyword::Match => "MATCH",
            Keyword::Merge => "MERGE",
            Keyword::Nan => "NAN",
            Keyword::None => "NONE",
            Keyword::Not => "NOT",
            Keyword::Null => "NULL",
            Keyword::Offset => "OFFSET",
            Keyword::On => "ON",
            Keyword::Optional => "OPTIONAL",
            Keyword::Or => "OR",
            Keyword::Order => "ORDER",
            Keyword::Path => "PATH",
            Keyword::Paths => "PATHS",
            Keyword::Reduce => "REDUCE",
            Keyword::Remove => "REMOVE",
            Keyword::Return => "RETURN",
            Keyword::Set => "SET",
            Keyword::Shortest => "SHORTEST",
            Keyword::ShortestPath => "SHORTESTPATH",
            Keyword::Single => "SINGLE",
            Keyword::Skip => "SKIP",
            Keyword::Starts => "STARTS",
            Keyword::Then => "THEN",
            Keyword::Trim => "TRIM",
            Keyword::True => "TRUE",
            Keyword::Union => "UNION",
            Keyword::Unwind => "UNWIND",
            Keyword::When => "WHEN",
            Keyword::Where => "WHERE",
            Keyword::With => "WITH",
            Keyword::Xor => "XOR",
            Keyword::Yield => "YIELD",
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Null,
    Boolean(bool),
    String(String),
    Integer(u64),
    Float(f64),
    /// `INF` / `INFINITY`, with an optional leading sign.
    Infinity {
        negative: bool,
    },
    Nan,
    /// `<list literal>`: elements are literals, not expressions.
    List(Vec<Literal>),
}

impl Literal {
    pub fn try_from(token: token::TokenKind) -> Result<Self, errors::Error> {
        match token {
            token::TokenKind::Keyword(token::Keyword::Null) => Ok(Self::Null),
            token::TokenKind::Keyword(token::Keyword::True) => Ok(Self::Boolean(true)),
            token::TokenKind::Keyword(token::Keyword::False) => Ok(Self::Boolean(false)),
            token::TokenKind::Identifier(value) => Ok(Self::String(value)),
            token::TokenKind::Integer(value) => Ok(Self::Integer(value)),
            token::TokenKind::Float(value) => Ok(Self::Float(value)),
            token::TokenKind::String(value) => Ok(Self::String(value)),
            _ => Err(errors::Error::UnexpectedToken {
                location: "Literal::try_from".to_string(),
                token,
            }),
        }
    }
}

// A reference to a property on a variable (`p.age` in `MATCH (p:Person) WHERE p.age > 10`)
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyReference {
    pub variable: Variable,
    pub property: Property,
}

// Expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    AdvancedComparison {
        op: AdvancedComparisonOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    /// `OR`, `XOR`, `AND`, and the arithmetic operators.
    Binary {
        op: BinaryOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Case {
        whens: Box<Expression>,
        else_: Box<Expression>,
    },
    Comparison {
        lhs: Box<Expression>,
        op: ComparisonOp,
        rhs: Box<Expression>,
    },
    CountStar,
    Identifier(String),
    Literal(Literal),
    PropertyReference(PropertyReference),
    /// `NOT`, unary `+`, unary `-`.
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::Display, strum::EnumString,
)]
pub enum AdvancedComparisonOp {
    #[strum(serialize = "CONTAINS")]
    Contains,
    #[strum(serialize = "IN")]
    In,
    #[strum(serialize = "=~")]
    RegexEqual,
    #[strum(serialize = "STARTS WITH")]
    StartsWith,
    #[strum(serialize = "ENDS WITH")]
    EndsWith,
}

impl AdvancedComparisonOp {
    pub const fn binding_power(self) -> u8 {
        9
    }
}

impl TryFrom<token::Token> for AdvancedComparisonOp {
    type Error = strum::ParseError;

    fn try_from(value: token::Token) -> Result<Self, Self::Error> {
        match value.kind {
            token::TokenKind::Keyword(token::Keyword::Contains) => Ok(Self::Contains),
            token::TokenKind::Keyword(token::Keyword::In) => Ok(Self::In),
            token::TokenKind::Op(token::Op::EqTilde) => Ok(Self::RegexEqual),
            _ => Err(strum::ParseError::VariantNotFound),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::Display, strum::EnumString,
)]
pub enum BinaryOp {
    #[strum(serialize = "OR")]
    Or,
    #[strum(serialize = "XOR")]
    Xor,
    #[strum(serialize = "AND")]
    And,
    #[strum(serialize = "+")]
    Add,
    #[strum(serialize = "-")]
    Subtract,
    #[strum(serialize = "*")]
    Multiply,
    #[strum(serialize = "/")]
    Divide,
    #[strum(serialize = "%")]
    Modulo,
    #[strum(serialize = "^")]
    Power,
}

impl TryFrom<token::Token> for BinaryOp {
    type Error = strum::ParseError;

    fn try_from(value: token::Token) -> Result<Self, Self::Error> {
        match value.kind {
            token::TokenKind::Punct(token::Punct::And) => Ok(Self::And),
            token::TokenKind::Punct(token::Punct::Pipe) => Ok(Self::Or),
            other => {
                let strval = other.to_string();
                Self::try_from(strval.as_str())
            }
        }
    }
}

impl BinaryOp {
    pub const fn binding_power(self) -> u8 {
        match self {
            Self::Or => 1,
            Self::Xor => 3,
            Self::And => 5,
            Self::Add | Self::Subtract => 13,
            Self::Multiply | Self::Divide | Self::Modulo => 15,
            Self::Power => 19,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::Display, strum::EnumString,
)]
pub enum ComparisonOp {
    #[strum(serialize = "=")]
    Equal,
    #[strum(serialize = "<>")]
    NotEqual,
    #[strum(serialize = "<")]
    Less,
    #[strum(serialize = ">")]
    Greater,
    #[strum(serialize = "<=")]
    LessOrEqual,
    #[strum(serialize = ">=")]
    GreaterOrEqual,
}

impl ComparisonOp {
    pub const fn binding_power(self) -> u8 {
        11
    }
}

impl TryFrom<token::Token> for ComparisonOp {
    type Error = strum::ParseError;

    fn try_from(value: token::Token) -> Result<Self, Self::Error> {
        match value.kind {
            token::TokenKind::Op(token::Op::Eq) => Ok(Self::Equal),
            token::TokenKind::Op(token::Op::Ne) => Ok(Self::NotEqual),
            token::TokenKind::Op(token::Op::Lt) => Ok(Self::Less),
            token::TokenKind::Op(token::Op::Gt) => Ok(Self::Greater),
            token::TokenKind::Op(token::Op::Le) => Ok(Self::LessOrEqual),
            token::TokenKind::Op(token::Op::Ge) => Ok(Self::GreaterOrEqual),
            _ => Err(strum::ParseError::VariantNotFound),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::Display, strum::EnumString,
)]
pub enum UnaryOp {
    #[strum(serialize = "NOT")]
    Not,
    #[strum(serialize = "+")]
    Plus,
    #[strum(serialize = "-")]
    Minus,
}

impl TryFrom<token::Token> for UnaryOp {
    type Error = strum::ParseError;

    fn try_from(value: token::Token) -> Result<Self, Self::Error> {
        match &value.kind {
            token::TokenKind::Op(token::Op::Plus) => Ok(Self::Plus),
            token::TokenKind::Op(token::Op::Minus) => Ok(Self::Minus),
            token::TokenKind::Punct(token::Punct::Not) => Ok(Self::Not),
            token::TokenKind::Keyword(token::Keyword::Not) => Ok(Self::Not),
            _ => Err(strum::ParseError::VariantNotFound),
        }
    }
}

impl UnaryOp {
    pub const fn binding_power(self) -> u8 {
        17
    }
}
