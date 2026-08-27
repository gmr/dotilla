/// Cypher 9 Grammar Data Structures && Keywords

pub struct Statement {
    pub clause: Clause,
    pub modifiers: Vec<Modifier>,
    pub expressions: Vec<Expression>,
    pub functions: Vec<Function>,
}

pub enum Clause {
    call,
    create,
    cypher,
    delete,
    detach,
    exists,
    mandatory,
    r#match,
    merge,
    optional,
    remove,
    r#return,
    set,
    union,
    unwind,
    with,
    r#yield,
}

pub const COMMENT: &str = "//";
pub const COMMENT_STARTS: &str = "/*";
pub const COMMENT_ENDS: &str = "*/";

pub enum Modifier {
    asc,
    ascending,
    by,
    desc,
    descending,
    on,
}

pub enum Expression {
    all,
    case,
    r#else,
    end,
    then,
    when,
}

pub enum Function {
    abs,
    acos,
    asin,
    atan,
    atan20,
    avg,
    ceil,
    coalesce,
    collect,
    cos,
    cot,
    count,
    degrees,
    e,
    endNode,
    exists,
    exp,
    floor,
    head,
    id,
    keys,
    labels,
    last,
    left,
    length,
    log,
    log10,
    lTrim,
    max,
    min,
    nodes,
    percentileCount,
    percentileDisc,
    pi,
    properties,
    radians,
    rand,
    range,
    relationships,
    replace,
    reverse,
    right,
    round,
    rTrim,
    sign,
    sin,
    size,
    split,
    sqrt,
    startNode,
    stDev,
    stDevP,
    substring,
    sum,
    tail,
    tan,
    timestamp,
    toBoolean,
    toFloat,
    toInteger,
    toLower,
    toString,
    toUpper,
    trim,
    r#type,
}

pub enum Literal {
    r#false,
    null,
    r#true,
}

pub enum Operator {
    access,
    add,
    all,
    and,
    r#as,
    contains,
    distinct,
    divide,
    ends,
    equal,
    exponent,
    greaterThan,
    greaterThanOrEqual,
    r#in,
    inequal,
    is,
    lessThan,
    lessThanOrEqual,
    r#mod,
    multiply,
    not,
    or,
    starts,
    subtract,
    xor,
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
    add,
    contraint,
    r#do,
    drop,
    r#for,
    mandatory,
    of,
    require,
    scalar,
    unique,
}

pub enum Subclause {
    limit,
    order,
    skip,
    r#where,
}
