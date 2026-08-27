use std::str::FromStr;

use super::token::*;

pub struct Lexer {
    input: Box<str>,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.into(),
            position: 0,
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token.kind == TokenKind::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    pub fn next_token(&mut self) -> Token {
        let mut start = self.position;
        while let Some(byte) = self.peek() {
            start = self.position;

            // Try to handle comments
            match (byte, self.peek_at(1)) {
                (b'/', Some(b'/')) => {
                    self.skip_line_comment();
                    continue;
                }
                (b'/', Some(b'*')) => {
                    self.skip_block_comment();
                    continue;
                }
                _ => {}
            }

            // Try to handle operators
            if self.is_operator_start(byte) {
                if let Some(b2) = self.peek_at(1)
                    && let Ok(text) = std::str::from_utf8(&[byte, b2])
                    && let Ok(value) = Op::from_str(text)
                {
                    self.position += 2;
                    return Token {
                        kind: TokenKind::Op(value),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    };
                }
                if let Ok(text) = std::str::from_utf8(&[byte])
                    && let Ok(value) = Op::from_str(text)
                {
                    self.position += 1;
                    return Token {
                        kind: TokenKind::Op(value),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    };
                }
            }

            // Try to match punctuation
            if let Ok(value) = Punct::try_from(byte) {
                self.position += 1;
                return Token {
                    kind: TokenKind::Punct(value),
                    span: Span {
                        start,
                        end: self.position,
                    },
                };
            }

            // Try to handle keywords and identifiers
            if self.is_identifier_start(byte) {
                while self.peek().is_some_and(|b| self.is_identifier_continue(b)) {
                    self.position += 1;
                }
                match Keyword::from_str(&self.input[start..self.position]) {
                    Ok(value) => {
                        return Token {
                            kind: TokenKind::Keyword(value),
                            span: Span {
                                start,
                                end: self.position,
                            },
                        };
                    }
                    _ => {
                        return Token {
                            kind: TokenKind::Identifier(
                                self.input[start..self.position].to_string(),
                            ),
                            span: Span {
                                start,
                                end: self.position,
                            },
                        };
                    }
                }
            }

            // Handle numeric literals
            if byte.is_ascii_digit() {
                let mut is_float = false;
                while let Some(b) = self.peek()
                    && (b.is_ascii_digit() || b == b'.')
                {
                    if b == b'.' {
                        is_float = true;
                    }
                    self.position += 1;
                }
                if is_float {
                    let value: f64 = self.input[start..self.position].parse().unwrap();
                    return Token {
                        kind: TokenKind::Float(value),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    };
                }
                let value: i64 = self.input[start..self.position].parse().unwrap();
                return Token {
                    kind: TokenKind::Integer(value),
                    span: Span {
                        start,
                        end: self.position,
                    },
                };
            }

            if self.is_whitespace(byte) {
                self.position += 1;
            }
        }
        Token {
            kind: TokenKind::Eof,
            span: Span {
                start,
                end: self.position,
            },
        }
    }

    fn is_identifier_start(&self, byte: u8) -> bool {
        byte.is_ascii_alphabetic() || byte == b'_' || byte >= 0x80
    }

    fn is_identifier_continue(&self, byte: u8) -> bool {
        self.is_identifier_start(byte) || byte.is_ascii_digit()
    }

    fn is_operator_start(&self, byte: u8) -> bool {
        matches!(
            byte,
            b'=' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'^'
        )
    }

    fn is_whitespace(&self, byte: u8) -> bool {
        matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
    }

    fn peek(&self) -> Option<u8> {
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        let index = self.position.checked_add(offset)?;
        self.input.as_bytes().get(index).copied()
    }

    fn skip_block_comment(&mut self) {
        while let Some(byte) = self.peek() {
            if matches!(byte, b'*') && self.peek_at(1) == Some(b'/') {
                self.position += 2;
                break;
            }
            self.position += 1;
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(byte) = self.peek() {
            if matches!(byte, b'\n' | b'\r') {
                break;
            }
            self.position += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lexer = Lexer::new("MATCH (p:Person {age: 30}) RETURN p.name".to_string());
        let tokens = lexer.lex();
        println!("{:?}", tokens);
        //assert_eq!(tokens.len(), 16);
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Match));
        assert_eq!(tokens[1].kind, TokenKind::Punct(Punct::LParen));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[4].kind, TokenKind::Identifier("Person".to_string()));
        assert_eq!(tokens[5].kind, TokenKind::Punct(Punct::LBrace));
        assert_eq!(tokens[6].kind, TokenKind::Identifier("age".to_string()));
        assert_eq!(tokens[7].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[8].kind, TokenKind::Integer(30));
        assert_eq!(tokens[9].kind, TokenKind::Punct(Punct::RBrace));
        assert_eq!(tokens[10].kind, TokenKind::Punct(Punct::RParen));
        assert_eq!(tokens[11].kind, TokenKind::Keyword(Keyword::Return));
        assert_eq!(tokens[12].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[13].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[14].kind, TokenKind::Identifier("name".to_string()));
        assert_eq!(tokens[15].kind, TokenKind::Eof);
    }
}
