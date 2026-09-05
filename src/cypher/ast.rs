#[derive(Debug, Clone, PartialEq)]
pub enum Program {
    Procedure(CompositeStatement),
    StandaloneCall(StandaloneProcedureCall),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositeStatement {
    pub head: LinearStatement,
    pub tail: Vec<UnionArm>,
}

/// One `UNION [ ALL | DISTINCT ] <linear statement>`.
#[derive(Debug, Clone, PartialEq)]
pub struct UnionArm {
    pub quantifier: Option<SetQuantifier>,
    pub statement: LinearStatement,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearStatement {
    pub statements: Vec<PrimitiveStatement>,
    pub result: Option<ReturnStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveStatement {
    Match(MatchStatement),
    Unwind(UnwindStatement),
    With(WithStatement),
    Create(CreateStatement),
    Merge(MergeStatement),
    Set(SetStatement),
    Remove(RemoveStatement),
    Delete(DeleteStatement),
    Call(NamedProcedureCall),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SetQuantifier {
    All,
    Distinct,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStatement {
    pub optional: bool,
    pub pattern: GraphPattern,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnwindStatement {
    pub expression: Expr,
    pub variable: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithStatement {
    pub body: ReturnBody,
    pub order_and_page: Option<OrderByAndPage>,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    pub body: ReturnBody,
    pub order_and_page: Option<OrderByAndPage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnBody {
    pub quantifier: Option<SetQuantifier>,
    pub star: bool,
    pub items: Vec<ReturnItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnItem {
    pub expression: Expr,
    pub alias: Option<Ident>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OrderByAndPage {
    pub order_by: Vec<SortSpecification>,
    pub offset: Option<Expr>,
    pub limit: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortSpecification {
    pub key: Expr,
    pub order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateStatement {
    pub patterns: Vec<WritePathPattern>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergeStatement {
    pub pattern: WritePathPattern,
    pub action: Option<MergeAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergeAction {
    pub trigger: MergeTrigger,
    pub set: SetStatement,
}

/// `ON MATCH` / `ON CREATE`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MergeTrigger {
    Match,
    Create,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetStatement {
    pub items: Vec<SetItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetItem {
    /// `v = expr`
    AllProperties { target: Ident, value: Expr },
    /// `v += expr`
    AddAllProperties { target: Ident, value: Expr },
    /// `v:A:B`
    Labels { target: Ident, labels: Vec<Ident> },
    /// `expr.key = expr`, where `target` is a postfix expression.
    Property { target: Expr, value: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveStatement {
    pub items: Vec<RemoveItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveItem {
    Labels { target: Ident, labels: Vec<Ident> },
    Property(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub detach: bool,
    pub items: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedProcedureCall {
    pub procedure: ProcedureReference,
    pub arguments: Vec<Expr>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StandaloneProcedureCall {
    pub procedure: ProcedureReference,
    pub arguments: Option<Vec<Expr>>,
    pub yield_clause: Option<StandaloneYieldClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldClause {
    pub items: Vec<YieldItem>,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StandaloneYieldClause {
    /// `YIELD *`
    All,
    /// Non-empty.
    Items(Vec<YieldItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldItem {
    pub name: Ident,
    pub alias: Option<Ident>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureReference {
    /// `<catalog object parent reference>`, outermost first.
    pub namespace: Vec<Ident>,
    pub name: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionReference {
    pub namespace: Vec<Ident>,
    pub name: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphPattern {
    pub paths: Vec<PathPattern>,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathPattern {
    /// `p = ...`
    pub variable: Option<Ident>,
    pub prefix: Option<PathSearchPrefix>,
    pub expression: PathPatternExpression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSearchPrefix {
    /// `ALL [ PATH | PATHS ]`
    All,
    /// `ANY [ k ] [ PATH | PATHS ]`
    Any { count: Option<UnsignedInteger> },
    /// `ALL SHORTEST [ PATH | PATHS ]`
    AllShortest,
    /// `ANY SHORTEST [ PATH | PATHS ]`
    AnyShortest,
    /// `SHORTEST k [ PATH | PATHS ]`
    ShortestPaths { count: UnsignedInteger },
    /// `SHORTEST [ k ] [ PATH | PATHS ] { GROUP | GROUPS }`
    ShortestGroups { count: Option<UnsignedInteger> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnsignedInteger {
    Literal(u64),
    Parameter(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathPatternExpression {
    Concatenation(Vec<PathFactor>),
    Legacy(Box<LegacyShortestPath>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathFactor {
    pub primary: PathPrimary,
    pub quantifier: Option<PatternQuantifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathPrimary {
    Element(ElementPattern),
    Parenthesized(Box<ParenthesizedPathPattern>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParenthesizedPathPattern {
    pub variable: Option<Ident>,
    pub expression: PathPatternExpression,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyShortestPath {
    pub kind: LegacyShortestKind,
    pub start: NodePattern,
    pub relationship: RelationshipPattern,
    pub end: NodePattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyShortestKind {
    Shortest,
    AllShortest,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternQuantifier {
    ZeroOrMore,
    OneOrMore,
    Fixed(u64),
    Range {
        lower: Option<u64>,
        upper: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimplePathPattern {
    pub start: NodePattern,
    pub steps: Vec<PathStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathStep {
    pub relationship: RelationshipPattern,
    pub node: NodePattern,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementPattern {
    Node(NodePattern),
    Relationship(RelationshipPattern),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NodePattern {
    pub variable: Option<Ident>,
    pub label: Option<LabelExpression>,
    pub predicate: Option<ElementPredicate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationshipPattern {
    pub direction: Direction,
    /// `None` when the `[...]` bracket is absent entirely.
    pub detail: Option<RelationshipDetail>,
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RelationshipDetail {
    pub variable: Option<Ident>,
    pub label: Option<LabelExpression>,
    pub length: Option<PathLength>,
    pub predicate: Option<ElementPredicate>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathLength {
    /// `*`
    Any,
    /// `*n`
    Fixed(u64),
    /// `*m..n`, either bound optional.
    Range {
        lower: Option<u64>,
        upper: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementPredicate {
    /// `WHERE expr`
    Where(Box<Expr>),
    /// `{ k: v, ... }`
    Properties(Vec<PropertyKeyValuePair>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyKeyValuePair {
    pub key: Ident,
    pub value: Expr,
}

/// Data update patterns (CREATE and MERGE)
#[derive(Debug, Clone, PartialEq)]
pub struct WritePathPattern {
    pub variable: Option<Ident>,
    pub start: WriteNodePattern,
    pub steps: Vec<WritePathStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WritePathStep {
    pub relationship: WriteRelationshipPattern,
    pub node: WriteNodePattern,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WriteNodePattern {
    pub variable: Option<Ident>,
    pub labels: Vec<Ident>,
    pub properties: Option<ElementProperties>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WriteRelationshipPattern {
    pub direction: WriteDirection,
    pub variable: Option<Ident>,
    /// Exactly one label is required when writing.
    pub label: Ident,
    pub properties: Option<ElementProperties>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WriteDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementProperties {
    Map(Vec<PropertyKeyValuePair>),
    Parameter(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabelExpression {
    Label(Ident),
    /// `%`
    Wildcard,
    /// `!expr`
    Not(Box<LabelExpression>),
    /// `lhs & rhs`
    And(Box<LabelExpression>, Box<LabelExpression>),
    /// `lhs | rhs`
    Or(Box<LabelExpression>, Box<LabelExpression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// `<binding variable reference>`
    Variable(Ident),
    /// `<general parameter reference>`, `$name`.
    Parameter(Ident),
    Literal(Literal),
    /// `OR`, `XOR`, `AND`, and the arithmetic operators.
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `NOT`, unary `+`, unary `-`.
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// `<comparison predicate>`
    Comparison(Box<Comparison>),
    /// `<postfix expression>`: property access, indexing, slicing.
    Postfix {
        operand: Box<Expr>,
        op: Box<PostfixOp>,
    },
    /// A bare pattern used as a predicate, `<pattern expression>`.
    Pattern(Box<SimplePathPattern>),
    /// `<shortest path expression>`
    ShortestPath(Box<LegacyShortestPath>),
    /// `CASE ... END`
    Case(Box<CaseExpression>),
    /// `COUNT(*)`
    CountStar,
    /// `EXISTS { ... }`
    Exists(Box<SubqueryArgument>),
    /// `v { ... }`
    MapProjection(Box<MapProjection>),
    /// `[ v IN expr WHERE p | e ]`
    ListComprehension(Box<ListComprehension>),
    /// `[ p = (a)-[r]->(b) WHERE p | e ]`
    PatternComprehension(Box<PatternComprehension>),
    /// `REDUCE(acc = init, v IN list | step)`
    Reduce(Box<ReduceExpression>),
    /// `ALL`/`ANY`/`SINGLE`/`NONE`` (v IN list WHERE p)`
    Quantifier(Box<QuantifierExpression>),
    /// `TRIM(expr)`
    Trim(Box<Expr>),
    /// `<function invocation>`
    Function(Box<FunctionInvocation>),
    /// `[ a, b, c ]`
    List(Vec<Expr>),
    /// `{ a: 1, b: 2 }`
    Map(Vec<Field>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Or,
    Xor,
    And,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

impl BinaryOp {
    pub const fn binding_power(self) -> u8 {
        match self {
            Self::Or => 1,
            Self::Xor => 3,
            Self::And => 5,
            Self::Add | Self::Subtract => 9,
            Self::Multiply | Self::Divide | Self::Modulo => 11,
            Self::Power => 13,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub first: ComparisonPredicand,
    pub rest: Vec<ComparisonPart>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonPredicand {
    pub operand: Expr,
    pub advanced: Option<AdvancedComparison>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonPart {
    pub op: ComparisonOp,
    pub operand: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdvancedComparison {
    /// `IN`, `CONTAINS`, `=~`, `STARTS WITH`, `ENDS WITH`.
    Op {
        op: AdvancedCompOp,
        operand: Box<Expr>,
    },
    /// `IS [ NOT ] NULL`
    IsNull { negated: bool },
    /// `IS <label expression>` / `: <label expression>`
    IsLabeled(LabelExpression),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

impl ComparisonOp {
    pub const fn binding_power(self) -> u8 {
        7
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdvancedCompOp {
    Contains,
    In,
    RegexEqual,
    StartsWith,
    EndsWith,
}

impl AdvancedCompOp {
    pub const fn binding_power(self) -> u8 {
        15
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOp {
    /// `.name`
    Property(Ident),
    /// `[expr]`
    Index(Expr),
    /// `[from..to]`, either bound optional.
    Slice {
        from: Option<Expr>,
        to: Option<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseExpression {
    pub operand: Option<Expr>,
    pub whens: Vec<WhenClause>,
    pub else_result: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenClause {
    pub operands: Vec<Expr>,
    pub result: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubqueryArgument {
    Procedure(CompositeStatement),
    Pattern(GraphPattern),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapProjection {
    pub variable: Ident,
    pub elements: Vec<MapProjectionElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapProjectionElement {
    /// `key: expr`
    Literal { key: Ident, value: Expr },
    /// `.name`
    Field(Ident),
    /// `v`
    Variable(Ident),
    /// `.*`
    AllFields,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListComprehension {
    pub variable: Ident,
    pub source: Expr,
    pub filter: Option<Expr>,
    pub projection: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternComprehension {
    pub variable: Option<Ident>,
    pub pattern: SimplePathPattern,
    pub filter: Option<Expr>,
    pub projection: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReduceExpression {
    pub accumulator: Ident,
    pub initial: Expr,
    pub variable: Ident,
    pub source: Expr,
    pub step: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuantifierExpression {
    pub quantifier: Quantifier,
    pub variable: Ident,
    pub source: Expr,
    pub predicate: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quantifier {
    All,
    Any,
    Single,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionInvocation {
    pub function: FunctionReference,
    /// `count(DISTINCT x)`
    pub quantifier: Option<SetQuantifier>,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Null,
    Boolean(bool),
    String(String),
    Integer(i64),
    Float(f64),
    /// `INF` / `INFINITY`, with an optional leading sign.
    Infinity {
        negative: bool,
    },
    Nan,
    /// `<list literal>`: elements are literals, not expressions.
    List(Vec<Literal>),
    /// `<map literal>`: values are literals, not expressions.
    Map(Vec<FieldLiteral>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldLiteral {
    pub name: Ident,
    pub value: Literal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ident(pub String);

impl Ident {
    pub fn new(name: impl Into<String>) -> Self {
        Ident(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Ident {
    fn from(name: &str) -> Self {
        Ident(name.to_owned())
    }
}

impl From<String> for Ident {
    fn from(name: String) -> Self {
        Ident(name)
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for Ident {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

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
