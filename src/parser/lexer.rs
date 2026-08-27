use std::str::FromStr;

use super::errors::Error;
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

    pub fn lex(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token.kind == TokenKind::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    pub fn next_token(&mut self) -> Result<Token, Error> {
        while let Some(byte) = self.peek() {
            let start = self.position;

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
                    return Ok(Token {
                        kind: TokenKind::Op(value),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    });
                }
                if let Ok(text) = std::str::from_utf8(&[byte])
                    && let Ok(value) = Op::from_str(text)
                {
                    self.position += 1;
                    return Ok(Token {
                        kind: TokenKind::Op(value),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    });
                }
            }

            // Try to match punctuation
            if let Ok(value) = Punct::try_from(byte) {
                self.position += 1;
                return Ok(Token {
                    kind: TokenKind::Punct(value),
                    span: Span {
                        start,
                        end: self.position,
                    },
                });
            }

            // Handle backtick Identifiers
            if byte == b'`' {
                self.position += 1;
                loop {
                    match self.peek() {
                        Some(b'`') if self.peek_at(1) == Some(b'`') => self.position += 2,
                        Some(b'`') => {
                            self.position += 1;
                            break;
                        }
                        Some(_) => self.position += 1,
                        None => {
                            return Err(Error::UnterminatedIdentifier {
                                span: Span {
                                    start,
                                    end: self.position,
                                },
                            });
                        }
                    }
                }
                let name = self.input[start + 1..self.position - 1].replace("``", "`");
                return Ok(Token {
                    kind: TokenKind::Identifier(name),
                    span: Span {
                        start,
                        end: self.position,
                    },
                });
            }

            // Try to handle keywords and non-quoted identifiers
            if self.is_identifier_start(byte) {
                while self.peek().is_some_and(|b| self.is_identifier_continue(b)) {
                    self.position += 1;
                }
                match Keyword::from_str(&self.input[start..self.position]) {
                    Ok(value) => {
                        return Ok(Token {
                            kind: TokenKind::Keyword(value),
                            span: Span {
                                start,
                                end: self.position,
                            },
                        });
                    }
                    _ => {
                        return Ok(Token {
                            kind: TokenKind::Identifier(
                                self.input[start..self.position].to_string(),
                            ),
                            span: Span {
                                start,
                                end: self.position,
                            },
                        });
                    }
                }
            }

            // Handle Parameters
            if byte == b'$' {
                self.position += 1;
                while self
                    .peek()
                    .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    self.position += 1;
                }
                return Ok(Token {
                    kind: TokenKind::Parameter(self.input[start + 1..self.position].to_string()),
                    span: Span {
                        start,
                        end: self.position,
                    },
                });
            }

            // Handle string values
            if self.is_quote(byte) {
                self.position += 1;
                let quote = byte;
                let mut escaped = false;
                loop {
                    let current = match self.peek() {
                        Some(current) => {
                            self.position += 1;
                            current
                        }
                        None => {
                            return Err(Error::UnterminatedString {
                                span: Span {
                                    start,
                                    end: self.position,
                                },
                            });
                        }
                    };
                    if escaped {
                        escaped = false;
                    } else if current == b'\\' {
                        escaped = true;
                    } else if current == quote {
                        break;
                    }
                }
                return Ok(Token {
                    kind: TokenKind::String(self.input[start + 1..self.position - 1].to_string()),
                    span: Span {
                        start,
                        end: self.position,
                    },
                });
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
                if is_float && let Ok(value) = self.input[start..self.position].parse::<f64>() {
                    return Ok(Token {
                        kind: TokenKind::Float(value),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    });
                }
                if let Ok(value) = self.input[start..self.position].parse::<i64>() {
                    return Ok(Token {
                        kind: TokenKind::Integer(value),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    });
                };
            }

            if self.is_whitespace(byte) {
                self.position += 1;
                continue;
            }

            return Err(Error::UnexpectedByte {
                byte,
                span: Span {
                    start,
                    end: start + 1,
                },
            });
        }
        Ok(Token {
            kind: TokenKind::Eof,
            span: Span {
                start: self.position,
                end: self.position,
            },
        })
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

    fn is_quote(&self, byte: u8) -> bool {
        matches!(byte, b'"' | b'\'')
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
        let mut lexer = Lexer::new(
            "MATCH (p:Person {age: 30, gender: $gender, income > 123.45})
                WHERE p.fname = \"Ralph\"
                  AND p.mname = \"\\\"Ralphie\\\"\"
                  AND p.lname = \"Wiggum\"
                  AND p.`is a` = \"Fire Engine\"
                RETURN p.name"
                .to_string(),
        );
        let Ok(tokens) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        println!("{:?}", tokens);
        assert_eq!(tokens.len(), 48);
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Match));
        assert_eq!(tokens[1].kind, TokenKind::Punct(Punct::LParen));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[4].kind, TokenKind::Identifier("Person".to_string()));
        assert_eq!(tokens[5].kind, TokenKind::Punct(Punct::LBrace));
        assert_eq!(tokens[6].kind, TokenKind::Identifier("age".to_string()));
        assert_eq!(tokens[7].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[8].kind, TokenKind::Integer(30));
        assert_eq!(tokens[9].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[10].kind, TokenKind::Identifier("gender".to_string()));
        assert_eq!(tokens[11].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[12].kind, TokenKind::Parameter("gender".to_string()));
        assert_eq!(tokens[13].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[14].kind, TokenKind::Identifier("income".to_string()));
        assert_eq!(tokens[15].kind, TokenKind::Op(Op::Gt));
        assert_eq!(tokens[16].kind, TokenKind::Float(123.45));
        assert_eq!(tokens[17].kind, TokenKind::Punct(Punct::RBrace));
        assert_eq!(tokens[18].kind, TokenKind::Punct(Punct::RParen));
        assert_eq!(tokens[19].kind, TokenKind::Keyword(Keyword::Where));
        assert_eq!(tokens[20].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[21].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[22].kind, TokenKind::Identifier("fname".to_string()));
        assert_eq!(tokens[23].kind, TokenKind::Op(Op::Eq));
        assert_eq!(tokens[24].kind, TokenKind::String("Ralph".to_string()));
        assert_eq!(tokens[25].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[26].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[27].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[28].kind, TokenKind::Identifier("mname".to_string()));
        assert_eq!(tokens[29].kind, TokenKind::Op(Op::Eq));
        assert_eq!(
            tokens[30].kind,
            TokenKind::String("\\\"Ralphie\\\"".to_string())
        );
        assert_eq!(tokens[31].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[32].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[33].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[34].kind, TokenKind::Identifier("lname".to_string()));
        assert_eq!(tokens[35].kind, TokenKind::Op(Op::Eq));
        assert_eq!(tokens[36].kind, TokenKind::String("Wiggum".to_string()));
        assert_eq!(tokens[37].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[38].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[39].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[40].kind, TokenKind::Identifier("is a".to_string()));
        assert_eq!(tokens[41].kind, TokenKind::Op(Op::Eq));
        assert_eq!(
            tokens[42].kind,
            TokenKind::String("Fire Engine".to_string())
        );
        assert_eq!(tokens[43].kind, TokenKind::Keyword(Keyword::Return));
        assert_eq!(tokens[44].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[45].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[46].kind, TokenKind::Identifier("name".to_string()));
        assert_eq!(tokens[47].kind, TokenKind::Eof);
    }
}
