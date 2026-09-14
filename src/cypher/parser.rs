#![allow(unused)]

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
        if self.peek_n(0).kind != token::TokenKind::Eof {
            return Err(errors::Error::UnexpectedToken {
                location: "end of query".to_string(),
                token: self.peek_n(0).kind,
            });
        }
        Ok(query)
    }

    fn parse_query(&mut self) -> Result<ast::Query, errors::Error> {
        Err(errors::Error::NotImplemented)
    }
}

// Other parsing
impl Parser {}

// Expression Parsing
impl Parser {
    /// Parses an expression with the given minimum binding power.
    fn parse_expression(&mut self, min_bp: u8) -> Result<ast::Expression, errors::Error> {
        let mut lhs: ast::Expression;
        if let Some(op) = self.parse_unary_operator() {
            self.advance(1);
            let rhs = self.parse_expression(op.binding_power() + 1)?;
            lhs = ast::Expression::Unary {
                op,
                operand: Box::new(rhs),
            };
        } else if let Some((value, advance)) = self.parse_atom() {
            self.advance(advance);
            lhs = value;
        } else {
            return Err(errors::Error::UnexpectedToken {
                location: "expression".to_string(),
                token: self.peek_n(0).kind,
            });
        }

        loop {
            if let Some((op, advance)) = self.parse_advanced_comparison_operator() {
                if op.binding_power() < min_bp {
                    break;
                }
                self.advance(advance);
                let rhs = self.parse_expression(op.binding_power() + 1)?;
                lhs = ast::Expression::AdvancedComparison {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                continue;
            }
            if let Some(op) = self.parse_binary_operator() {
                if op.binding_power() < min_bp {
                    break;
                }
                self.advance(1);
                let rhs = self.parse_expression(op.binding_power() + 1)?;
                lhs = ast::Expression::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                continue;
            }
            if let Some(op) = self.parse_comparison_operator() {
                if op.binding_power() < min_bp {
                    break;
                }
                self.advance(1);
                let rhs = self.parse_expression(op.binding_power() + 1)?;
                lhs = ast::Expression::Comparison {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };
                continue;
            }
            break;
        }
        Ok(lhs)
    }

    /// Parses an advanced comparison operator, if one is present.
    fn parse_advanced_comparison_operator(&mut self) -> Option<(ast::AdvancedComparisonOp, usize)> {
        match ast::AdvancedComparisonOp::try_from(self.peek_n(0)) {
            Ok(op) => Some((op, 1)),
            Err(_) => {
                let p1 = self.peek_n(0);
                let p2 = self.peek_n(1);
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
        match self.peek_n(0).kind.clone() {
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
        ast::BinaryOp::try_from(self.peek_n(0)).ok()
    }

    /// Parses a comparison operator, if one is present.
    fn parse_comparison_operator(&mut self) -> Option<ast::ComparisonOp> {
        ast::ComparisonOp::try_from(self.peek_n(0)).ok()
    }

    /// Parses a unary operator, if one is present.
    fn parse_unary_operator(&mut self) -> Option<ast::UnaryOp> {
        ast::UnaryOp::try_from(self.peek_n(0)).ok()
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
        self.peek_n(0).kind == token::TokenKind::Eof || self.position >= self.tokens.len()
    }

    /// Skips the current token if it is a keyword token with the given keyword.
    fn maybe_skip_keyword_token(&mut self, keyword: token::Keyword) {
        if self.peek_n(0).kind == token::TokenKind::Keyword(keyword) {
            self.advance(1);
        }
    }

    /// Parses a property reference, if one is present, if Some, you need to advance 3
    fn parse_property_reference(&mut self) -> Option<ast::PropertyReference> {
        let p1 = self.peek_n(0);
        let p2 = self.peek_n(1);
        let p3 = self.peek_n(2);
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
    fn peek_n(&self, offset: usize) -> token::Token {
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

    // Parsing Expression Test Case

    #[test]
    fn test_parse_expression_case_1() {
        let mut parser = Parser::new(vec![
            new_identifier_token("p"),
            new_punct_token(token::Punct::Dot),
            new_identifier_token("name"),
            new_op_token(token::Op::Eq),
            new_string_token("Fred".to_string()),
            new_eof_token(),
        ]);
        match parser.parse_expression(0) {
            Ok(result) => {
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
            Err(err) => panic!("{:?}", err),
        }
    }

    #[test]
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
        match parser.parse_expression(0) {
            Ok(result) => {
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
            Err(err) => panic!("{:?}", err),
        }
    }

    // Utility Function Tests

    #[test]
    fn test_advance() {
        let mut parser = fixture_node_parser();
        assert_eq!(parser.position, 0);
        parser.advance(1);
        assert_eq!(parser.position, 1);

        assert_eq!(
            parser.peek_n(0).kind,
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
    fn test_peek() {
        let parser = fixture_node_parser();
        assert_eq!(parser.position, 0);
        assert_eq!(
            parser.peek_n(0).kind,
            token::TokenKind::Punct(token::Punct::LParen)
        );
    }

    #[test]
    fn test_peek_n() {
        let parser = fixture_node_parser();
        assert_eq!(parser.position, 0);
        assert_eq!(
            parser.peek_n(1).kind,
            token::TokenKind::Identifier("p".to_string()),
        );
        assert_eq!(parser.peek_n(20).kind, token::TokenKind::Eof,);
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
