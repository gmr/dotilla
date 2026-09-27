#![allow(unused)]
use std::collections::HashMap;

use super::{ast, errors, token};

pub fn parse(tokens: Vec<token::Token>) -> Result<ast::Query, errors::Error> {
    Parser::new(tokens).parse()
}

pub struct Parser {
    tokens: Vec<token::Token>,
    position: usize,
}

// Public Functions
impl Parser {
    pub fn new(tokens: Vec<token::Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn parse(mut self) -> Result<ast::Query, errors::Error> {
        let query = self.parse_query()?;
        if self.peek(0).kind != token::TokenKind::Eof {
            return Err(errors::Error::UnexpectedToken {
                location: "end of query".to_string(),
                token: self.peek(0).kind,
            });
        }
        Ok(query)
    }

    fn parse_query(&mut self) -> Result<ast::Query, errors::Error> {
        let mut clauses: Vec<ast::Clause> = Vec::new();
        let mut return_clause: Option<ast::Return> = None;
        let p1 = self.peek(1);
        let p2 = self.peek(2);
        match (p1.kind, p2.kind) {
            (token::TokenKind::Keyword(token::Keyword::Match), _) => {
                let clause = self.parse_match_statement()?;
                clauses.push(ast::Clause::Match(clause));
            }
            (
                token::TokenKind::Keyword(token::Keyword::Optional),
                token::TokenKind::Keyword(token::Keyword::Match),
            ) => {
                let clause = self.parse_match_statement()?;
                clauses.push(ast::Clause::Match(clause));
            }
            (token::TokenKind::Keyword(token::Keyword::Return), _) => {
                return_clause = self.parse_return_statement()?;
            }
            _ => {}
        }
        if clauses.is_empty() {
            Err(errors::Error::UnexpectedEof)
        } else {
            Ok(ast::Query {
                clauses,
                return_clause,
            })
        }
    }

    fn parse_match_statement(&mut self) -> Result<ast::Match, errors::Error> {
        let optional = self.peek(0).kind == token::TokenKind::Keyword(token::Keyword::Optional);
        if optional {
            self.advance(1); // OPTIONAL
        }
        self.advance(1); // MATCH
        Err(errors::Error::NotImplemented)
    }

    fn parse_return_statement(&mut self) -> Result<Option<ast::Return>, errors::Error> {
        Err(errors::Error::NotImplemented)
    }
}

// Path Parsing
impl Parser {
    fn parse_path(&mut self) -> Result<ast::Path, errors::Error> {
        let mut segments: Vec<ast::Segment> = vec![];
        while !self.is_eof() {
            if let Some(segment) = self.parse_path_segment()? {
                segments.push(segment);
            } else {
                break;
            }
        }
        Ok(ast::Path { segments })
    }

    fn parse_path_segment(&mut self) -> Result<Option<ast::Segment>, errors::Error> {
        let result = match self.peek(0).kind {
            token::TokenKind::Punct(token::Punct::LParen) => {
                let node = Some(self.parse_node()?);
                let direction = self.parse_direction()?;
                Some(ast::Segment {
                    node,
                    edge: None,
                    direction,
                })
            }
            token::TokenKind::Punct(token::Punct::LBracket) => {
                let edge = Some(self.parse_edge()?);
                let direction = self.parse_direction()?;
                Some(ast::Segment {
                    node: None,
                    edge,
                    direction,
                })
            }
            _ => None,
        };
        Ok(result)
    }

    fn parse_node(&mut self) -> Result<ast::Node, errors::Error> {
        if self.peek(0).kind != token::TokenKind::Punct(token::Punct::LParen) {
            return Err(errors::Error::UnexpectedToken {
                location: "parse_node (1)".to_string(),
                token: self.peek(0).kind.clone(),
            });
        }
        self.advance(1);
        let mut variable: Option<ast::Variable> = None;
        let mut labels: Vec<ast::Label> = vec![];
        let mut predicate: Option<ast::Predicate> = None;
        if let Some(value) = self.parse_relationship_detail(token::Punct::Colon, true)? {
            variable = value.variable;
            labels = value.labels;
        }
        if self.peek(0).kind == token::TokenKind::Punct(token::Punct::LBrace) {
            predicate = Some(self.parse_predicate()?);
        }
        if self.peek(0).kind != token::TokenKind::Punct(token::Punct::RParen) {
            return Err(errors::Error::UnexpectedToken {
                location: "parse_node (2)".to_string(),
                token: self.peek(0).kind.clone(),
            });
        }
        self.advance(1);
        Ok(ast::Node {
            variable,
            labels,
            predicate,
        })
    }

    fn parse_edge(&mut self) -> Result<ast::Edge, errors::Error> {
        let ob = self.peek(0);
        if ob.kind != token::TokenKind::Punct(token::Punct::LBracket) {
            return Err(errors::Error::UnexpectedToken {
                location: "parse_edge".to_string(),
                token: ob.kind.clone(),
            });
        } else {
            self.advance(1);
        }

        let mut labels: Vec<ast::Label> = vec![];
        let mut length: Option<ast::PathLength> = None;
        let mut predicate: Option<ast::Predicate> = None;
        let mut variable: Option<ast::Variable> = None;

        if let Some(value) = self.parse_relationship_detail(token::Punct::Pipe, false)? {
            variable = value.variable;
            labels = value.labels;
        }

        while !self.is_eof() {
            match self.peek(0).kind {
                // Variable Path Length old syntax `*1..5`
                token::TokenKind::Op(token::Op::Star) => {
                    length = self.parse_deprecated_path_length()?;
                    continue;
                }
                // Predicates `{key: 'value', ...}`)
                token::TokenKind::Punct(token::Punct::LBrace) => {
                    predicate = Some(self.parse_predicate()?);
                    continue;
                }
                token::TokenKind::Punct(token::Punct::RBracket) => {
                    self.advance(1);
                    break;
                }
                (value) => {
                    return Err(errors::Error::UnexpectedToken {
                        location: "parse_edge".to_string(),
                        token: value.clone(),
                    });
                }
            }
        }
        Ok(ast::Edge {
            variable,
            labels,
            length,
            predicate,
        })
    }

    fn parse_relationship_detail(
        &mut self,
        delimiter: token::Punct,
        allow_wildcards: bool,
    ) -> Result<Option<ast::RelationshipDetail>, errors::Error> {
        let variable: Option<ast::Variable> = match (self.peek(0).kind, self.peek(1).kind) {
            (token::TokenKind::Identifier(value), token::TokenKind::Punct(token::Punct::Colon)) => {
                self.advance(1);
                Some(ast::Variable(value.to_string()))
            }
            (token::TokenKind::Identifier(value), var) => {
                self.advance(1);
                Some(ast::Variable(value.to_string()))
            }
            _ => None,
        };
        if self.peek(0).kind != token::TokenKind::Punct(token::Punct::Colon) {
            return Ok(Some(ast::RelationshipDetail {
                variable,
                labels: vec![],
            }));
        }
        self.advance(1);
        let mut labels: Vec<ast::Label> = vec![];
        while !self.is_eof() {
            match self.peek(0).kind {
                token::TokenKind::Identifier(value) => {
                    labels.push(ast::Label(value.to_string()));
                    self.advance(1);
                }
                token::TokenKind::Op(token::Op::Percent) if allow_wildcards => {
                    labels.push(ast::Label("%".to_string()));
                    self.advance(1);
                }
                token::TokenKind::Punct(value) if value == delimiter => {
                    self.advance(1);
                    continue;
                }
                _ => break,
            }
        }
        Ok(Some(ast::RelationshipDetail { variable, labels }))
    }

    // Parse deprecated variable path length syntax `[*1..5]`
    fn parse_deprecated_path_length(&mut self) -> Result<Option<ast::PathLength>, errors::Error> {
        match (
            self.peek(0).kind,
            self.peek(1).kind,
            self.peek(2).kind,
            self.peek(3).kind,
        ) {
            (
                token::TokenKind::Op(token::Op::Star),
                token::TokenKind::Integer(lower),
                token::TokenKind::Punct(token::Punct::DotDot),
                token::TokenKind::Integer(upper),
            ) => {
                self.advance(4);
                Ok(Some(ast::PathLength::Range {
                    lower: Some(lower),
                    upper: Some(upper),
                }))
            }
            (
                token::TokenKind::Op(token::Op::Star),
                token::TokenKind::Integer(lower),
                token::TokenKind::Punct(token::Punct::DotDot),
                _,
            ) => {
                self.advance(3);
                Ok(Some(ast::PathLength::Range {
                    lower: Some(lower),
                    upper: None,
                }))
            }
            (
                token::TokenKind::Op(token::Op::Star),
                token::TokenKind::Punct(token::Punct::DotDot),
                token::TokenKind::Integer(upper),
                _,
            ) => {
                self.advance(3);
                Ok(Some(ast::PathLength::Range {
                    lower: None,
                    upper: Some(upper),
                }))
            }
            (token::TokenKind::Op(token::Op::Star), token::TokenKind::Integer(value), _, _) => {
                self.advance(3);
                Ok(Some(ast::PathLength::Fixed(value)))
            }
            (token::TokenKind::Op(token::Op::Star), _, _, _) => Ok(Some(ast::PathLength::Any)),
            _ => Ok(None),
        }
    }

    // Parse Modern path length syntax `{1,3}` after a path node
    fn parse_modern_path_length(&mut self) -> Result<Option<ast::PathLength>, errors::Error> {
        match (
            self.peek(0).kind,
            self.peek(1).kind,
            self.peek(2).kind,
            self.peek(3).kind,
            self.peek(4).kind,
        ) {
            (
                token::TokenKind::Punct(token::Punct::LBrace),
                token::TokenKind::Integer(lower),
                token::TokenKind::Punct(token::Punct::Comma),
                token::TokenKind::Integer(upper),
                token::TokenKind::Punct(token::Punct::RBrace),
            ) => {
                self.advance(5);
                Ok(Some(ast::PathLength::Range {
                    lower: Some(lower),
                    upper: Some(upper),
                }))
            }
            (
                token::TokenKind::Punct(token::Punct::LBrace),
                token::TokenKind::Integer(lower),
                token::TokenKind::Punct(token::Punct::Comma),
                token::TokenKind::Punct(token::Punct::RBrace),
                _,
            ) => {
                self.advance(4);
                Ok(Some(ast::PathLength::Range {
                    lower: Some(lower),
                    upper: None,
                }))
            }
            (
                token::TokenKind::Punct(token::Punct::LBrace),
                token::TokenKind::Punct(token::Punct::Comma),
                token::TokenKind::Integer(upper),
                token::TokenKind::Punct(token::Punct::RBrace),
                _,
            ) => {
                self.advance(4);
                Ok(Some(ast::PathLength::Range {
                    lower: None,
                    upper: Some(upper),
                }))
            }
            (
                token::TokenKind::Punct(token::Punct::LBrace),
                token::TokenKind::Integer(value),
                token::TokenKind::Punct(token::Punct::RBrace),
                _,
                _,
            ) => {
                self.advance(3);
                Ok(Some(ast::PathLength::Fixed(value)))
            }
            _ => Ok(None),
        }
    }

    // Parse an element's kv properties predicate ({k:v ...})
    fn parse_predicate(&mut self) -> Result<ast::Predicate, errors::Error> {
        match self.peek(0).kind {
            token::TokenKind::Punct(token::Punct::LBrace) => {
                let pairs = self.parse_properties()?;
                Ok(ast::Predicate {
                    properties: Some(pairs),
                    where_clause: None,
                })
            }
            _ => Err(errors::Error::UnexpectedToken {
                location: "parse_predicate".to_string(),
                token: self.peek(0).kind.clone(),
            }),
        }
    }

    // Parse an ea query WHERE predicate
    fn parse_where(&mut self) -> Result<ast::Predicate, errors::Error> {
        match self.peek(0).kind {
            token::TokenKind::Keyword(token::Keyword::Where) => {
                self.advance(1);
                let expr = self.parse_expression(0)?;
                Ok(ast::Predicate {
                    properties: None,
                    where_clause: Some(expr),
                })
            }
            _ => Err(errors::Error::UnexpectedToken {
                location: "parse_predicate".to_string(),
                token: self.peek(0).kind.clone(),
            }),
        }
    }

    /// Parses a property key-value pair, e.g. `{k: v, ...}`.
    fn parse_properties(&mut self) -> Result<HashMap<ast::Property, ast::Literal>, errors::Error> {
        let open = self.peek(0);
        if open.kind != token::TokenKind::Punct(token::Punct::LBrace) {
            return Err(errors::Error::UnexpectedToken {
                location: "parse_property_key_value_pairs (1)".to_string(),
                token: open.kind.clone(),
            });
        }
        self.advance(1);

        let mut properties: HashMap<ast::Property, ast::Literal> = HashMap::new();
        while !self.is_eof() {
            match (self.peek(0).kind, self.peek(1).kind, self.peek(2).kind) {
                (token::TokenKind::Punct(token::Punct::RBrace), _, _) => {
                    self.advance(1);
                    return Ok(properties);
                }
                (token::TokenKind::Punct(token::Punct::Comma), _, _) => {
                    self.advance(1);
                    continue;
                }
                (
                    token::TokenKind::Identifier(key),
                    token::TokenKind::Punct(token::Punct::Colon),
                    value,
                ) => {
                    let value = ast::Literal::try_from(value)?;
                    self.advance(3);
                    properties.insert(ast::Property(key), value);
                }
                (kind, _, _) => {
                    return Err(errors::Error::UnexpectedToken {
                        location: "parse_property_key_value_pairs (2)".to_string(),
                        token: kind,
                    });
                }
            }
        }
        Err(errors::Error::UnexpectedToken {
            location: "parse_properties_eof".to_string(),
            token: token::TokenKind::Eof,
        })
    }

    fn parse_direction(&mut self) -> Result<Option<ast::Direction>, errors::Error> {
        match (self.peek(0).kind, self.peek(1).kind) {
            (token::TokenKind::Op(token::Op::Lt), token::TokenKind::Op(token::Op::Minus)) => {
                self.advance(2);
                Ok(Some(ast::Direction::Left))
            }
            (token::TokenKind::Op(token::Op::Minus), token::TokenKind::Op(token::Op::Gt)) => {
                self.advance(2);
                Ok(Some(ast::Direction::Right))
            }
            (token::TokenKind::Op(token::Op::Minus), _) => {
                self.advance(2);
                Ok(Some(ast::Direction::Undirected))
            }
            _ => Ok(None),
        }
    }
}

// Expression Parsing
impl Parser {
    /// Parses an expression with the given minimum binding power.
    fn parse_expression(&mut self, min_bp: u8) -> Result<ast::Expression, errors::Error> {
        let mut lhs: Option<ast::Expression> = None;

        if let Some(op) = self.parse_unary_operator() {
            self.advance(1);
            let rhs = self.parse_expression(op.binding_power() + 1)?;
            lhs = Some(ast::Expression::Unary {
                op,
                operand: Box::new(rhs),
            });
        } else if let Some((value, advance)) = self.parse_atom() {
            self.advance(advance);
            lhs = Some(value);
        } else if let Some(expr) = self.parse_count_star() {
            self.advance(4);
            lhs = Some(expr);
        }

        loop {
            if let Some((op, advance)) = self.parse_advanced_comparison_operator()
                && lhs.is_some()
            {
                if op.binding_power() < min_bp {
                    break;
                }
                self.advance(advance);
                let rhs = self.parse_expression(op.binding_power() + 1)?;
                lhs = Some(ast::Expression::AdvancedComparison {
                    op,
                    lhs: Box::new(lhs.unwrap()),
                    rhs: Box::new(rhs),
                });
                continue;
            }
            if let Some(op) = self.parse_binary_operator()
                && lhs.is_some()
            {
                if op.binding_power() < min_bp {
                    break;
                }
                self.advance(1);
                let rhs = self.parse_expression(op.binding_power() + 1)?;
                lhs = Some(ast::Expression::Binary {
                    op,
                    lhs: Box::new(lhs.unwrap()),
                    rhs: Box::new(rhs),
                });
                continue;
            }
            if let Some(op) = self.parse_comparison_operator()
                && lhs.is_some()
            {
                if op.binding_power() < min_bp {
                    break;
                }
                self.advance(1);
                let rhs = self.parse_expression(op.binding_power() + 1)?;
                lhs = Some(ast::Expression::Comparison {
                    op,
                    lhs: Box::new(lhs.unwrap()),
                    rhs: Box::new(rhs),
                });
                continue;
            }
            break;
        }
        if let Some(item) = lhs {
            Ok(item)
        } else {
            Err(errors::Error::UnexpectedToken {
                location: "parse_expression".to_string(),
                token: self.peek(0).kind,
            })
        }
    }

    /// Parses an advanced comparison operator, if one is present.
    fn parse_advanced_comparison_operator(&mut self) -> Option<(ast::AdvancedComparisonOp, usize)> {
        match ast::AdvancedComparisonOp::try_from(self.peek(0)) {
            Ok(op) => Some((op, 1)),
            Err(_) => {
                let p1 = self.peek(0);
                let p2 = self.peek(1);
                match (p1.kind, p2.kind) {
                    (
                        token::TokenKind::Keyword(token::Keyword::Starts),
                        token::TokenKind::Keyword(token::Keyword::With),
                    ) => Some((ast::AdvancedComparisonOp::StartsWith, 2)),
                    (
                        token::TokenKind::Keyword(token::Keyword::Ends),
                        token::TokenKind::Keyword(token::Keyword::With),
                    ) => Some((ast::AdvancedComparisonOp::EndsWith, 2)),
                    _ => None,
                }
            }
        }
    }

    /// Parses an atom, such as an integer, float, or string, or a property reference (a.foo)
    fn parse_atom(&mut self) -> Option<(ast::Expression, usize)> {
        match self.peek(0).kind.clone() {
            token::TokenKind::Integer(value) => {
                Some((ast::Expression::Literal(ast::Literal::Integer(value)), 1))
            }
            token::TokenKind::Float(value) => {
                Some((ast::Expression::Literal(ast::Literal::Float(value)), 1))
            }
            token::TokenKind::String(value) => {
                Some((ast::Expression::Literal(ast::Literal::String(value)), 1))
            }
            token::TokenKind::Identifier(_) => {
                self.parse_property_reference().map(|property_reference| {
                    (ast::Expression::PropertyReference(property_reference), 3)
                })
            }
            _ => None,
        }
    }

    /// Parses a binary operator, if one is present.
    fn parse_binary_operator(&mut self) -> Option<ast::BinaryOp> {
        ast::BinaryOp::try_from(self.peek(0)).ok()
    }

    /// Parses a comparison operator, if one is present.
    fn parse_comparison_operator(&mut self) -> Option<ast::ComparisonOp> {
        ast::ComparisonOp::try_from(self.peek(0)).ok()
    }

    /// Parses a `COUNT(*)` expression, if one is present.
    fn parse_count_star(&mut self) -> Option<ast::Expression> {
        if self.peek(0).kind == token::TokenKind::Keyword(token::Keyword::Count)
            && self.peek(1).kind == token::TokenKind::Punct(token::Punct::LParen)
            && self.peek(2).kind == token::TokenKind::Op(token::Op::Star)
            && self.peek(3).kind == token::TokenKind::Punct(token::Punct::RParen)
        {
            Some(ast::Expression::CountStar)
        } else {
            None
        }
    }

    /// Parses a unary operator, if one is present.
    fn parse_unary_operator(&mut self) -> Option<ast::UnaryOp> {
        ast::UnaryOp::try_from(self.peek(0)).ok()
    }
}

// Utility Functions
impl Parser {
    /// Advances the parser position by the given count, if possible.
    fn advance(&mut self, count: usize) {
        if self.position < self.tokens.len() {
            self.position += count;
        }
    }

    /// Returns true if the current token is an `Eof` token or we hit the end of the token stack.
    fn is_eof(&self) -> bool {
        self.peek(0).kind == token::TokenKind::Eof || self.position >= self.tokens.len()
    }

    /// Skips the current token if it is a keyword token with the given keyword.
    fn maybe_skip_keyword_token(&mut self, keyword: token::Keyword) {
        if self.peek(0).kind == token::TokenKind::Keyword(keyword) {
            self.advance(1);
        }
    }

    /// Parses a property reference, if one is present, if Some, you need to advance 3
    fn parse_property_reference(&mut self) -> Option<ast::PropertyReference> {
        let p1 = self.peek(0);
        let p2 = self.peek(1);
        let p3 = self.peek(2);
        match (p1.kind, p2.kind, p3.kind) {
            (
                token::TokenKind::Identifier(variable),
                token::TokenKind::Punct(token::Punct::Dot),
                token::TokenKind::Identifier(property),
            ) => Some(ast::PropertyReference {
                variable: ast::Variable(variable),
                property: ast::Property(property),
            }),
            _ => None,
        }
    }

    /// Returns the current token, or an `Eof` token if the end of the tokens has been reached.
    fn peek(&self, offset: usize) -> token::Token {
        let token = match self.tokens.get(self.position + offset) {
            Some(token) => token,
            None => self.tokens.last().unwrap(),
        };
        token.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cypher::lexer;

    #[test]
    fn test_parse_path_case_1() {
        let tokens = lexer::lex("(a:Foo)->[b:BAR]->(c:Baz)").unwrap();
        let mut parser = Parser::new(tokens);
        let result = parser.parse_path();
        assert!(result.is_ok());
        let result = result.unwrap();
        let expectation = ast::Path {
            segments: vec![
                ast::Segment {
                    node: Some(ast::Node {
                        variable: Some(ast::Variable("a".to_string())),
                        labels: vec![ast::Label("Foo".to_string())],
                        predicate: None,
                    }),
                    direction: Some(ast::Direction::Right),
                    edge: None,
                },
                ast::Segment {
                    edge: Some(ast::Edge {
                        variable: Some(ast::Variable("b".to_string())),
                        labels: vec![ast::Label("BAR".to_string())],
                        length: None,
                        predicate: None,
                    }),
                    direction: Some(ast::Direction::Right),
                    node: None,
                },
                ast::Segment {
                    node: Some(ast::Node {
                        variable: Some(ast::Variable("c".to_string())),
                        labels: vec![ast::Label("Baz".to_string())],
                        predicate: None,
                    }),
                    direction: None,
                    edge: None,
                },
            ],
        };
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_path_case_2() {
        let tokens = lexer::lex("(a:Foo) RETURN a").unwrap();
        let mut parser = Parser::new(tokens);
        let result = parser.parse_path();
        assert!(result.is_ok());
        let result = result.unwrap();
        let expectation = ast::Path {
            segments: vec![ast::Segment {
                node: Some(ast::Node {
                    variable: Some(ast::Variable("a".to_string())),
                    labels: vec![ast::Label("Foo".to_string())],
                    predicate: None,
                }),
                direction: None,
                edge: None,
            }],
        };
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_node_case_1() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LParen),
            new_identifier_token("a"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("BAR"),
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_punct_token(token::Punct::RBrace),
            new_punct_token(token::Punct::RParen),
            new_eof_token(),
        ]);
        let mut properties: HashMap<ast::Property, ast::Literal> = HashMap::new();
        properties.insert(
            ast::Property("foo".to_string()),
            ast::Literal::String("bar".to_string()),
        );
        let expectation = ast::Node {
            variable: Some(ast::Variable("a".to_string())),
            labels: vec![ast::Label("FOO".to_string()), ast::Label("BAR".to_string())],
            predicate: Some(ast::Predicate {
                properties: Some(properties),
                where_clause: None,
            }),
        };
        let result = parser.parse_node();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_node_case_2() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("a"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_eof_token(),
        ]);
        let err = parser.parse_node().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_node_case_3() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LParen),
            new_identifier_token("a"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);
        let err = parser.parse_node().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_edge_case_1() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBracket),
            new_identifier_token("a"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_op_token(token::Op::Star),
            new_integer_token(1),
            new_punct_token(token::Punct::DotDot),
            new_integer_token(5),
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_punct_token(token::Punct::RBrace),
            new_punct_token(token::Punct::RBracket),
            new_eof_token(),
        ]);
        let mut properties: HashMap<ast::Property, ast::Literal> = HashMap::new();
        properties.insert(
            ast::Property("foo".to_string()),
            ast::Literal::String("bar".to_string()),
        );
        let expectation = ast::Edge {
            variable: Some(ast::Variable("a".to_string())),
            labels: vec![ast::Label("FOO".to_string())],
            predicate: Some(ast::Predicate {
                properties: Some(properties),
                where_clause: None,
            }),
            length: Some(ast::PathLength::Range {
                lower: Some(1),
                upper: Some(5),
            }),
        };
        let result = parser.parse_edge();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_edge_case_2() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("a"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_eof_token(),
        ]);
        let err = parser.parse_edge().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_edge_case_3() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBracket),
            new_identifier_token("a"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_op_token(token::Op::Star),
            new_integer_token(1),
            new_punct_token(token::Punct::DotDot),
            new_integer_token(5),
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_punct_token(token::Punct::RBrace),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);
        let err = parser.parse_edge().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_edge_case_4() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBracket),
            new_op_token(token::Op::Star),
            new_integer_token(1),
            new_punct_token(token::Punct::DotDot),
            new_integer_token(5),
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_punct_token(token::Punct::RBrace),
            new_punct_token(token::Punct::RBracket),
            new_eof_token(),
        ]);
        let mut properties: HashMap<ast::Property, ast::Literal> = HashMap::new();
        properties.insert(
            ast::Property("foo".to_string()),
            ast::Literal::String("bar".to_string()),
        );
        let expectation = ast::Edge {
            variable: None,
            labels: vec![],
            predicate: Some(ast::Predicate {
                properties: Some(properties),
                where_clause: None,
            }),
            length: Some(ast::PathLength::Range {
                lower: Some(1),
                upper: Some(5),
            }),
        };
        let result = parser.parse_edge();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_edge_case_5() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBracket),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_op_token(token::Op::Star),
            new_integer_token(1),
            new_punct_token(token::Punct::DotDot),
            new_integer_token(5),
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_punct_token(token::Punct::RBrace),
            new_punct_token(token::Punct::RBracket),
            new_eof_token(),
        ]);
        let mut properties: HashMap<ast::Property, ast::Literal> = HashMap::new();
        properties.insert(
            ast::Property("foo".to_string()),
            ast::Literal::String("bar".to_string()),
        );
        let expectation = ast::Edge {
            variable: None,
            labels: vec![ast::Label("FOO".to_string())],
            predicate: Some(ast::Predicate {
                properties: Some(properties),
                where_clause: None,
            }),
            length: Some(ast::PathLength::Range {
                lower: Some(1),
                upper: Some(5),
            }),
        };
        let result = parser.parse_edge();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_edge_case_6() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBracket),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_punct_token(token::Punct::RBrace),
            new_punct_token(token::Punct::RBracket),
            new_eof_token(),
        ]);
        let mut properties: HashMap<ast::Property, ast::Literal> = HashMap::new();
        properties.insert(
            ast::Property("foo".to_string()),
            ast::Literal::String("bar".to_string()),
        );
        let expectation = ast::Edge {
            variable: None,
            labels: vec![ast::Label("FOO".to_string())],
            predicate: Some(ast::Predicate {
                properties: Some(properties),
                where_clause: None,
            }),
            length: None,
        };
        let result = parser.parse_edge();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_edge_case_7() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBracket),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::RBracket),
            new_eof_token(),
        ]);
        let expectation = ast::Edge {
            variable: None,
            labels: vec![ast::Label("FOO".to_string())],
            predicate: None,
            length: None,
        };
        let result = parser.parse_edge();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_edge_case_8() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBracket),
            new_punct_token(token::Punct::RBracket),
            new_eof_token(),
        ]);
        let expectation = ast::Edge {
            variable: None,
            labels: vec![],
            predicate: None,
            length: None,
        };
        let result = parser.parse_edge();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_relationship_detail_case_1() {
        let mut parser = Parser::new(vec![
            new_identifier_token("a"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::RBracket),
            new_eof_token(),
        ]);
        let expectation = Some(ast::RelationshipDetail {
            variable: Some(ast::Variable("a".to_string())),
            labels: vec![ast::Label("FOO".to_string())],
        });
        let result = parser.parse_relationship_detail(token::Punct::Pipe, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_relationship_detail_case_2() {
        let mut parser = Parser::new(vec![
            new_identifier_token("a"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::Pipe),
            new_identifier_token("BAR"),
            new_eof_token(),
        ]);
        let expectation = Some(ast::RelationshipDetail {
            variable: Some(ast::Variable("a".to_string())),
            labels: vec![ast::Label("FOO".to_string()), ast::Label("BAR".to_string())],
        });
        let result = parser.parse_relationship_detail(token::Punct::Pipe, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_relationship_detail_case_3() {
        let mut parser = Parser::new(vec![
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::Pipe),
            new_identifier_token("BAR"),
            new_eof_token(),
        ]);
        let expectation = Some(ast::RelationshipDetail {
            variable: Some(ast::Variable("FOO".to_string())),
            labels: vec![],
        });
        let result = parser.parse_relationship_detail(token::Punct::Pipe, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_relationship_detail_case_4() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::Pipe),
            new_identifier_token("BAR"),
            new_eof_token(),
        ]);
        let expectation = Some(ast::RelationshipDetail {
            variable: None,
            labels: vec![ast::Label("FOO".to_string()), ast::Label("BAR".to_string())],
        });
        let result = parser.parse_relationship_detail(token::Punct::Pipe, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_relationship_detail_case_5() {
        let mut parser = Parser::new(vec![new_identifier_token("a"), new_eof_token()]);
        let expectation = Some(ast::RelationshipDetail {
            variable: Some(ast::Variable("a".to_string())),
            labels: vec![],
        });
        let result = parser.parse_relationship_detail(token::Punct::Pipe, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_relationship_detail_case_6() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::Colon),
            new_identifier_token("FOO"),
            new_punct_token(token::Punct::Colon),
            new_op_token(token::Op::Percent),
            new_eof_token(),
        ]);
        let expectation = Some(ast::RelationshipDetail {
            variable: None,
            labels: vec![ast::Label("FOO".to_string()), ast::Label("%".to_string())],
        });
        let result = parser.parse_relationship_detail(token::Punct::Colon, true);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_deprecated_path_length_case_1() {
        let mut parser = Parser::new(vec![new_op_token(token::Op::Star), new_eof_token()]);
        let expectation = Some(ast::PathLength::Any);
        let result = parser.parse_deprecated_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_deprecated_path_length_case_2() {
        let mut parser = Parser::new(vec![
            new_op_token(token::Op::Star),
            new_integer_token(2),
            new_punct_token(token::Punct::DotDot),
            new_integer_token(9),
            new_eof_token(),
        ]);
        let expectation = Some(ast::PathLength::Range {
            lower: Some(2),
            upper: Some(9),
        });
        let result = parser.parse_deprecated_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_deprecated_path_length_case_3() {
        let mut parser = Parser::new(vec![
            new_op_token(token::Op::Star),
            new_integer_token(2),
            new_punct_token(token::Punct::DotDot),
            new_eof_token(),
        ]);
        let expectation = Some(ast::PathLength::Range {
            lower: Some(2),
            upper: None,
        });
        let result = parser.parse_deprecated_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_deprecated_path_length_case_4() {
        let mut parser = Parser::new(vec![
            new_op_token(token::Op::Star),
            new_punct_token(token::Punct::DotDot),
            new_integer_token(4),
            new_eof_token(),
        ]);
        let expectation = Some(ast::PathLength::Range {
            lower: None,
            upper: Some(4),
        });
        let result = parser.parse_deprecated_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_deprecated_path_length_case_5() {
        let mut parser = Parser::new(vec![
            new_op_token(token::Op::Star),
            new_integer_token(4),
            new_eof_token(),
        ]);
        let expectation = Some(ast::PathLength::Fixed(4));
        let result = parser.parse_deprecated_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_deprecated_path_length_case_6() {
        let mut parser = Parser::new(vec![
            new_op_token(token::Op::Minus),
            new_integer_token(4),
            new_eof_token(),
        ]);
        let result = parser.parse_deprecated_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_modern_path_length_case_1() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_integer_token(2),
            new_punct_token(token::Punct::Comma),
            new_integer_token(9),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);
        let expectation = Some(ast::PathLength::Range {
            lower: Some(2),
            upper: Some(9),
        });
        let result = parser.parse_modern_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_modern_path_length_case_2() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_integer_token(2),
            new_punct_token(token::Punct::Comma),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);
        let expectation = Some(ast::PathLength::Range {
            lower: Some(2),
            upper: None,
        });
        let result = parser.parse_modern_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_modern_path_length_case_3() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_punct_token(token::Punct::Comma),
            new_integer_token(4),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);
        let expectation = Some(ast::PathLength::Range {
            lower: None,
            upper: Some(4),
        });
        let result = parser.parse_modern_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_modern_path_length_case_4() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_integer_token(42),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);
        let expectation = Some(ast::PathLength::Fixed(42));
        let result = parser.parse_modern_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_modern_path_length_case_5() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LParen),
            new_integer_token(42),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);
        let result = parser.parse_modern_path_length();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_predicate_case_1() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_punct_token(token::Punct::Comma),
            new_identifier_token("baz"),
            new_punct_token(token::Punct::Colon),
            new_integer_token(42),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);

        let mut properties: HashMap<ast::Property, ast::Literal> = HashMap::new();
        properties.insert(
            ast::Property("foo".to_string()),
            ast::Literal::String("bar".to_string()),
        );
        properties.insert(ast::Property("baz".to_string()), ast::Literal::Integer(42));
        let expectation = ast::Predicate {
            properties: Some(properties),
            where_clause: None,
        };
        let result = parser.parse_predicate();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_predicate_case_2() {
        let mut parser = Parser::new(vec![new_punct_token(token::Punct::LParen), new_eof_token()]);
        let err = parser.parse_predicate().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_where_case_1() {
        let mut parser = Parser::new(vec![
            new_keyword_token(token::Keyword::Where),
            new_identifier_token("a"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("foo"),
            new_op_token(token::Op::Eq),
            new_string_token("bar".to_string()),
            new_keyword_token(token::Keyword::And),
            new_identifier_token("a"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("baz"),
            new_op_token(token::Op::Eq),
            new_integer_token(42),
            new_eof_token(),
        ]);
        let expr = ast::Expression::Binary {
            op: ast::BinaryOp::And,
            lhs: Box::new(ast::Expression::Comparison {
                lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                    variable: ast::Variable("a".to_string()),
                    property: ast::Property("foo".to_string()),
                })),
                op: ast::ComparisonOp::Equal,
                rhs: Box::new(ast::Expression::Literal(ast::Literal::String(
                    "bar".to_string(),
                ))),
            }),
            rhs: Box::new(ast::Expression::Comparison {
                lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                    variable: ast::Variable("a".to_string()),
                    property: ast::Property("baz".to_string()),
                })),
                op: ast::ComparisonOp::Equal,
                rhs: Box::new(ast::Expression::Literal(ast::Literal::Integer(42))),
            }),
        };
        let expectation = ast::Predicate {
            properties: None,
            where_clause: Some(expr),
        };
        let result = parser.parse_where();
        println!("{:?}", result);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_where_case_2() {
        let mut parser = Parser::new(vec![new_punct_token(token::Punct::LParen), new_eof_token()]);
        let err = parser.parse_where().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_properties_case_1() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_punct_token(token::Punct::Comma),
            new_identifier_token("baz"),
            new_punct_token(token::Punct::Colon),
            new_integer_token(42),
            new_punct_token(token::Punct::RBrace),
            new_eof_token(),
        ]);

        let mut expectation: HashMap<ast::Property, ast::Literal> = HashMap::new();
        expectation.insert(
            ast::Property("foo".to_string()),
            ast::Literal::String("bar".to_string()),
        );
        expectation.insert(ast::Property("baz".to_string()), ast::Literal::Integer(42));
        let result = parser.parse_properties();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn test_parse_properties_case_2() {
        let mut parser = Parser::new(vec![new_punct_token(token::Punct::LParen), new_eof_token()]);
        let err = parser.parse_properties().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_properties_case_3() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_punct_token(token::Punct::LParen),
            new_eof_token(),
        ]);
        let err = parser.parse_properties().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_properties_case_4() {
        let mut parser = Parser::new(vec![
            new_punct_token(token::Punct::LBrace),
            new_identifier_token("foo"),
            new_punct_token(token::Punct::Colon),
            new_string_token("bar".to_string()),
            new_eof_token(),
        ]);
        let err = parser.parse_properties().unwrap_err();
        assert!(
            matches!(err, errors::Error::UnexpectedToken { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_relationship_direction_case_1() {
        let mut parser = Parser::new(vec![
            new_op_token(token::Op::Minus),
            new_op_token(token::Op::Gt),
            new_eof_token(),
        ]);
        let result = parser.parse_direction();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, Some(ast::Direction::Right));
    }

    #[test]
    fn test_parse_relationship_direction_case_2() {
        let mut parser = Parser::new(vec![
            new_op_token(token::Op::Lt),
            new_op_token(token::Op::Minus),
            new_eof_token(),
        ]);
        let result = parser.parse_direction();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, Some(ast::Direction::Left));
    }

    #[test]
    fn test_parse_relationship_direction_case_3() {
        let mut parser = Parser::new(vec![
            new_op_token(token::Op::Minus),
            new_punct_token(token::Punct::LParen),
            new_eof_token(),
        ]);
        let result = parser.parse_direction();
        let result = result.unwrap();
        assert_eq!(result, Some(ast::Direction::Undirected));
    }

    #[test]
    fn test_parse_relationship_direction_case_4() {
        let mut parser = Parser::new(vec![new_punct_token(token::Punct::LParen), new_eof_token()]);
        let result = parser.parse_direction();
        let result = result.unwrap();
        assert_eq!(result, None);
    }

    #[test]
    /// Property Reference Equality Comparison to String Literal
    fn test_parse_expression_case_1() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("name"),
            new_op_token(token::Op::Eq),
            new_string_token("Fred".to_string()),
            new_eof_token(),
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result,
            ast::Expression::Comparison {
                lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                    variable: ast::Variable("p".to_string()),
                    property: ast::Property("name".to_string()),
                })),
                op: ast::ComparisonOp::Equal,
                rhs: Box::new(ast::Expression::Literal(ast::Literal::String(
                    "Fred".to_string()
                ))),
            }
        );
    }

    #[test]
    /// Property Reference >= Comparison to Numeric Literal
    fn test_parse_expression_case_2() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("age"),
            new_op_token(token::Op::Ge),
            new_op_token(token::Op::Minus),
            token::Token {
                kind: token::TokenKind::Integer(5),
                span: token::Span::default(),
            },
            new_eof_token(),
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result,
            ast::Expression::Comparison {
                lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                    variable: ast::Variable("p".to_string()),
                    property: ast::Property("age".to_string()),
                })),
                op: ast::ComparisonOp::GreaterOrEqual,
                rhs: Box::new(ast::Expression::Unary {
                    op: ast::UnaryOp::Minus,
                    operand: Box::new(ast::Expression::Literal(ast::Literal::Integer(5))),
                }),
            }
        );
    }

    #[test]
    /// Count Star Expression
    fn test_parse_expression_case_3() {
        let mut parser = Parser::new(vec![
            new_keyword_token(token::Keyword::Count),
            new_punct_token(token::Punct::LParen),
            new_op_token(token::Op::Star),
            new_punct_token(token::Punct::RParen),
            new_eof_token(),
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, ast::Expression::CountStar);
    }

    #[test]
    /// Regex Match Expression
    fn test_parse_expression_case_4() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("name"),
            new_op_token(token::Op::EqTilde),
            token::Token {
                kind: token::TokenKind::String(".*".to_string()),
                span: token::Span::default(),
            },
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result,
            ast::Expression::AdvancedComparison {
                lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                    variable: ast::Variable("p".to_string()),
                    property: ast::Property("name".to_string()),
                })),
                op: ast::AdvancedComparisonOp::RegexEqual,
                rhs: Box::new(ast::Expression::Literal(ast::Literal::String(
                    ".*".to_string()
                ))),
            }
        );
    }

    #[test]
    /// Binary Operator Expression
    fn test_parse_expression_case_5() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("age"),
            new_op_token(token::Op::Plus),
            token::Token {
                kind: token::TokenKind::Float(4.2),
                span: token::Span::default(),
            },
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result,
            ast::Expression::Binary {
                lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                    variable: ast::Variable("p".to_string()),
                    property: ast::Property("age".to_string()),
                })),
                op: ast::BinaryOp::Add,
                rhs: Box::new(ast::Expression::Literal(ast::Literal::Float(4.2))),
            }
        );
    }

    #[test]
    /// Unexpected token
    fn test_parse_expression_case_6() {
        let mut parser = Parser::new(vec![
            new_keyword_token(token::Keyword::Where),
            new_eof_token(),
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, errors::Error::UnexpectedToken { .. }));
    }

    #[test]
    /// p.score * 2 + 1
    fn test_parse_expression_case_7() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("score"),
            new_op_token(token::Op::Star),
            token::Token {
                kind: token::TokenKind::Integer(2),
                span: token::Span::default(),
            },
            new_op_token(token::Op::Plus),
            token::Token {
                kind: token::TokenKind::Integer(1),
                span: token::Span::default(),
            },
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result,
            ast::Expression::Binary {
                lhs: Box::new(ast::Expression::Binary {
                    lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                        variable: ast::Variable("p".to_string()),
                        property: ast::Property("score".to_string()),
                    })),
                    op: ast::BinaryOp::Multiply,
                    rhs: Box::new(ast::Expression::Literal(ast::Literal::Integer(2))),
                }),
                op: ast::BinaryOp::Add,
                rhs: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
            }
        );
    }

    #[test]
    /// p.age + 1 >= 21
    fn test_parse_expression_case_8() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("age"),
            new_op_token(token::Op::Plus),
            token::Token {
                kind: token::TokenKind::Integer(1),
                span: token::Span::default(),
            },
            new_op_token(token::Op::Ge),
            token::Token {
                kind: token::TokenKind::Integer(21),
                span: token::Span::default(),
            },
            new_eof_token(),
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result,
            ast::Expression::Comparison {
                lhs: Box::new(ast::Expression::Binary {
                    lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                        variable: ast::Variable("p".to_string()),
                        property: ast::Property("age".to_string()),
                    })),
                    op: ast::BinaryOp::Add,
                    rhs: Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                }),
                op: ast::ComparisonOp::GreaterOrEqual,
                rhs: Box::new(ast::Expression::Literal(ast::Literal::Integer(21))),
            }
        );
    }

    #[test]
    /// p.name + "!" STARTS WITH "A"
    fn test_parse_expression_case_9() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("name"),
            new_op_token(token::Op::Plus),
            new_string_token("!".to_string()),
            new_keyword_token(token::Keyword::Starts),
            new_keyword_token(token::Keyword::With),
            new_string_token("A".to_string()),
            new_eof_token(),
        ]);
        let result = parser.parse_expression(0);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result,
            ast::Expression::AdvancedComparison {
                lhs: Box::new(ast::Expression::Binary {
                    lhs: Box::new(ast::Expression::PropertyReference(ast::PropertyReference {
                        variable: ast::Variable("p".to_string()),
                        property: ast::Property("name".to_string()),
                    })),
                    op: ast::BinaryOp::Add,
                    rhs: Box::new(ast::Expression::Literal(ast::Literal::String(
                        "!".to_string()
                    ))),
                }),
                op: ast::AdvancedComparisonOp::StartsWith,
                rhs: Box::new(ast::Expression::Literal(ast::Literal::String(
                    "A".to_string()
                ))),
            }
        );
    }

    #[test]
    fn test_parse_advanced_comparison_operator_starts_with() {
        let mut parser = Parser::new(vec![
            new_keyword_token(token::Keyword::Starts),
            new_keyword_token(token::Keyword::With),
            new_eof_token(),
        ]);
        assert!(parser.parse_advanced_comparison_operator().is_some());
        let (result, advance) = parser.parse_advanced_comparison_operator().unwrap();
        assert_eq!(advance, 2);
        assert_eq!(result, ast::AdvancedComparisonOp::StartsWith);
    }

    #[test]
    fn test_parse_advanced_comparison_operator_ends_with() {
        let mut parser = Parser::new(vec![
            new_keyword_token(token::Keyword::Ends),
            new_keyword_token(token::Keyword::With),
            new_eof_token(),
        ]);
        assert!(parser.parse_advanced_comparison_operator().is_some());
        let (result, advance) = parser.parse_advanced_comparison_operator().unwrap();
        assert_eq!(advance, 2);
        assert_eq!(result, ast::AdvancedComparisonOp::EndsWith);
    }

    #[test]
    fn test_parse_property_reference() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("age"),
            new_eof_token(),
        ]);
        let result = parser.parse_property_reference();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(
            result,
            ast::PropertyReference {
                variable: ast::Variable("p".to_string()),
                property: ast::Property("age".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_property_reference_none() {
        let mut parser = Parser::new(vec![new_identifier_token("p"), new_eof_token()]);
        assert!(parser.parse_property_reference().is_none());
    }

    // Utility Function Tests

    #[test]
    fn test_advance() {
        let mut parser = fixture_node_parser();
        assert_eq!(parser.position, 0);
        parser.advance(1);
        assert_eq!(parser.position, 1);
        assert_eq!(
            parser.peek(0).kind,
            token::TokenKind::Identifier("p".to_string()),
        );
    }

    #[test]
    fn test_is_eof() {
        let mut parser = fixture_node_parser();
        assert!(!parser.is_eof());
        parser.advance(parser.tokens.len() - 1);
        assert!(parser.is_eof());
    }

    #[test]
    fn test_maybe_skip_keyword_token() {
        let mut parser = Parser::new(vec![
            new_keyword_token(token::Keyword::Starts),
            new_keyword_token(token::Keyword::Ends),
            new_keyword_token(token::Keyword::With),
            new_eof_token(),
        ]);
        assert_eq!(parser.position, 0);
        parser.maybe_skip_keyword_token(token::Keyword::Ends);
        assert_eq!(parser.position, 0);
        parser.maybe_skip_keyword_token(token::Keyword::Starts);
        assert_eq!(parser.position, 1);
    }

    #[test]
    fn test_peek() {
        let parser = fixture_node_parser();
        assert_eq!(parser.position, 0);
        assert_eq!(
            parser.peek(1).kind,
            token::TokenKind::Identifier("p".to_string()),
        );
        assert_eq!(parser.peek(20).kind, token::TokenKind::Eof,);
    }

    // Test Fixture Functions

    fn fixture_node_parser() -> Parser {
        Parser::new(vec![
            new_punct_token(token::Punct::LParen),
            new_identifier_token("p"),
            new_punct_token(token::Punct::Colon),
            new_identifier_token("Person"),
            new_punct_token(token::Punct::RParen),
            new_eof_token(),
        ])
    }

    fn new_eof_token() -> token::Token {
        token::Token {
            kind: token::TokenKind::Eof,
            span: token::Span::default(),
        }
    }

    fn new_identifier_token(value: &str) -> token::Token {
        token::Token {
            kind: token::TokenKind::Identifier(value.to_string()),
            span: token::Span::default(),
        }
    }

    fn new_integer_token(value: u64) -> token::Token {
        token::Token {
            kind: token::TokenKind::Integer(value),
            span: token::Span::default(),
        }
    }

    fn new_keyword_token(value: token::Keyword) -> token::Token {
        token::Token {
            kind: token::TokenKind::Keyword(value),
            span: token::Span::default(),
        }
    }

    fn new_op_token(value: token::Op) -> token::Token {
        token::Token {
            kind: token::TokenKind::Op(value),
            span: token::Span::default(),
        }
    }

    fn new_string_token(value: String) -> token::Token {
        token::Token {
            kind: token::TokenKind::String(value.to_string()),
            span: token::Span::default(),
        }
    }

    fn new_punct_token(value: token::Punct) -> token::Token {
        token::Token {
            kind: token::TokenKind::Punct(value),
            span: token::Span::default(),
        }
    }
}
