// Cypher 9 Grammar Tokens

pub enum Clause {
    Call,
    Create,
    Cypher,
    Delete,
    Detach,
    Exists,
    Mandatory,
    Match,
    Merge,
    Optional,
    Remove,
    Return,
    Set,
    Union,
    Unwind,
    With,
    Yield,
}

pub const COMMENT: &str = "//";
pub const COMMENT_STARTS: &str = "/*";
pub const COMMENT_ENDS: &str = "*/";

pub enum Modifier {
    Asc,
    Ascending,
    By,
    Desc,
    Descending,
    On,
}

pub enum Expression {
    All,
    Case,
    Else,
    End,
    Then,
    When,
}

pub enum Literal {
    False,
    Null,
    True,
}

pub enum Operator {
    Access,
    Add,
    All,
    And,
    As,
    Contains,
    Distinct,
    Divide,
    Ends,
    Equal,
    Exponent,
    GreaterThan,
    GreaterThanOrEqual,
    In,
    Inequal,
    Is,
    LessThan,
    LessThanOrEqual,
    Mod,
    Multiply,
    Not,
    Or,
    Starts,
    Subtract,
    Xor,
}

pub const OPERATOR_ACCESS: &str = ".";
pub const OPERATOR_ADD: &str = "+";
pub const OPERATOR_DIVIDE: &str = "/";
pub const OPERATOR_EQUAL: &str = "=";
pub const OPERATOR_EXPONENT: &str = "^";
pub const OPERATOR_GREATER_THAN: &str = ">";
pub const OPERATOR_GREATER_THAN_OR_EQUAL: &str = ">=";
pub const OPERATOR_INEQUAL: &str = "<>";
pub const OPERATOR_LESS_THAN: &str = "<";
pub const OPERATOR_LESS_THAN_OR_EQUAL: &str = "<=";
pub const OPERATOR_MOD: &str = "%";
pub const OPERATOR_SUBTRACT: &str = "-";
pub const OPERATOR_MULTIPLY: &str = "*";

pub enum Reserved {
    Add,
    Constraint,
    Do,
    Drop,
    For,
    Mandatory,
    Of,
    Require,
    Scalar,
    Unique,
}

pub enum Subclause {
    Limit,
    Order,
    Skip,
    Where,
}
