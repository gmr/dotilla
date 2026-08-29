use std::str::FromStr;

use super::errors::Error;
use super::token::*;

pub fn lex(input: &str) -> Result<Vec<Token>, Error> {
    Lexer::new(input.to_string()).lex()
}

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
            if self.maybe_handle_comment(byte)? {
                continue;
            }
            if self.is_whitespace(byte) {
                self.position += 1;
                continue;
            } else if byte == b'0' && matches!(self.peek_at(1), Some(b'x') | Some(b'X')) {
                return self.handle_hex(start);
            } else if byte == b'0' && matches!(self.peek_at(1), Some(b'o') | Some(b'O')) {
                return self.handle_octal(start);
            } else if byte.is_ascii_digit()
                || (byte == b'.' && self.peek_at(1).is_some_and(|d| d.is_ascii_digit()))
            {
                return self.handle_number(start);
            } else if byte == b'`' {
                return self.handle_backtick_identifiers(start);
            } else if self.is_identifier_start(byte) {
                return self.handle_keyword_or_identifier(start);
            } else if let Some(token) = self.maybe_handle_operator_or_punct(byte, start) {
                return Ok(token);
            } else if byte == b'$' {
                return self.handle_parameters(start);
            } else if self.is_quote(byte) {
                return self.handle_string(byte, start);
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

    fn escaped_string_push(&mut self, value: &mut Vec<u8>, byte: u8) {
        value.push(byte);
        self.position += 1;
    }

    fn handle_backtick_identifiers(&mut self, start: usize) -> Result<Token, Error> {
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
        if (start + 1) == (self.position - 1) {
            Err(Error::InvalidIdentifier {
                span: Span {
                    start,
                    end: self.position,
                },
            })
        } else {
            let name = self.input[start + 1..self.position - 1].replace("``", "`");
            Ok(Token {
                kind: TokenKind::Identifier(name),
                span: Span {
                    start,
                    end: self.position,
                },
            })
        }
    }

    fn handle_hex(&mut self, start: usize) -> Result<Token, Error> {
        self.position += 2;
        while let Some(value) = self.peek() {
            if !value.is_ascii_hexdigit() {
                break;
            }
            self.position += 1;
        }
        match i64::from_str_radix(&self.input[start + 2..self.position], 16) {
            Ok(value) => Ok(Token {
                kind: TokenKind::Integer(value),
                span: Span {
                    start,
                    end: self.position,
                },
            }),
            Err(err) => Err(Error::ParseError {
                source: err,
                span: Span {
                    start,
                    end: self.position,
                },
            }),
        }
    }

    fn handle_keyword_or_identifier(&mut self, start: usize) -> Result<Token, Error> {
        while self.peek().is_some_and(|b| self.is_identifier_continue(b)) {
            self.position += 1;
        }
        match Keyword::from_str(&self.input[start..self.position]) {
            Ok(value) => Ok(Token {
                kind: TokenKind::Keyword(value),
                span: Span {
                    start,
                    end: self.position,
                },
            }),
            Err(_) => Ok(Token {
                kind: TokenKind::Identifier(self.input[start..self.position].to_string()),
                span: Span {
                    start,
                    end: self.position,
                },
            }),
        }
    }

    fn handle_number(&mut self, start: usize) -> Result<Token, Error> {
        let mut is_exponent = false;
        let mut is_float = false;
        while let Some(value) = self.peek() {
            if !is_exponent
                && matches!(value, b'e' | b'E')
                && self
                    .peek_at(1)
                    .is_some_and(|d| d.is_ascii_digit() || matches!(d, b'+' | b'-'))
            {
                is_exponent = true;
                is_float = true;
                self.position += 1;
                if matches!(self.peek(), Some(b'+' | b'-')) {
                    self.position += 1;
                    continue;
                }
            } else if !is_float
                && !is_exponent
                && matches!(value, b'.')
                && self.peek_at(1).is_some_and(|d| d.is_ascii_digit())
            {
                is_float = true;
                self.position += 1;
            } else if value.is_ascii_digit() {
                self.position += 1;
            } else {
                break;
            }
        }
        if is_float {
            match self.input[start..self.position].parse::<f64>() {
                Ok(value) if !value.is_finite() => Err(Error::NumberOutOfRange {
                    span: Span {
                        start,
                        end: self.position,
                    },
                }),
                Ok(value) => Ok(Token {
                    kind: TokenKind::Float(value),
                    span: Span {
                        start,
                        end: self.position,
                    },
                }),
                Err(err) => Err(Error::ParseFloatError {
                    source: err,
                    span: Span {
                        start,
                        end: self.position,
                    },
                }),
            }
        } else {
            match self.input[start..self.position].parse::<i64>() {
                Ok(value) => Ok(Token {
                    kind: TokenKind::Integer(value),
                    span: Span {
                        start,
                        end: self.position,
                    },
                }),
                Err(err) => Err(Error::ParseError {
                    source: err,
                    span: Span {
                        start,
                        end: self.position,
                    },
                }),
            }
        }
    }

    fn handle_octal(&mut self, start: usize) -> Result<Token, Error> {
        self.position += 2;
        while let Some(value) = self.peek() {
            if matches!(value, b'0'..=b'7') {
                self.position += 1;
            } else {
                break;
            }
        }
        match i64::from_str_radix(&self.input[start + 2..self.position], 8) {
            Ok(value) => Ok(Token {
                kind: TokenKind::Integer(value),
                span: Span {
                    start,
                    end: self.position,
                },
            }),
            Err(err) => Err(Error::ParseError {
                source: err,
                span: Span {
                    start,
                    end: self.position,
                },
            }),
        }
    }

    fn handle_parameters(&mut self, start: usize) -> Result<Token, Error> {
        self.position += 1;
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            self.position += 1;
        }
        if (start + 1) == self.position {
            Err(Error::InvalidParameter {
                span: Span {
                    start,
                    end: self.position,
                },
            })
        } else {
            Ok(Token {
                kind: TokenKind::Parameter(self.input[start + 1..self.position].to_string()),
                span: Span {
                    start,
                    end: self.position,
                },
            })
        }
    }

    fn handle_string(&mut self, byte: u8, start: usize) -> Result<Token, Error> {
        self.position += 1;
        let quote = byte;
        let mut value: Vec<u8> = Vec::new();
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
            if current == b'\\' {
                match self.peek() {
                    Some(b'b') => {
                        value.push(b'\x08');
                        self.position += 1
                    }
                    Some(b'f') => {
                        value.push(b'\x0c');
                        self.position += 1
                    }
                    Some(b'n') => self.escaped_string_push(&mut value, b'\n'),
                    Some(b'r') => self.escaped_string_push(&mut value, b'\r'),
                    Some(b't') => self.escaped_string_push(&mut value, b'\t'),
                    Some(b'\\') => self.escaped_string_push(&mut value, b'\\'),
                    Some(b'\'') => self.escaped_string_push(&mut value, b'\''),
                    Some(b'"') => self.escaped_string_push(&mut value, b'"'),
                    Some(b'u') => {
                        self.position += 1;
                        let n = self.position;
                        value.extend(self.hex_escape(
                            4,
                            Span {
                                start: n,
                                end: self.position,
                            },
                        )?);
                    }
                    Some(b'U') => {
                        self.position += 1;
                        let n = self.position;
                        value.extend(self.hex_escape(
                            8,
                            Span {
                                start: n,
                                end: self.position,
                            },
                        )?);
                    }
                    Some(_) => {
                        return Err(Error::InvalidEscape {
                            span: Span {
                                start,
                                end: self.position,
                            },
                        });
                    }
                    None => {
                        return Err(Error::UnterminatedString {
                            span: Span {
                                start,
                                end: self.position,
                            },
                        });
                    }
                }
            } else if current == quote {
                break;
            } else {
                value.push(current);
            }
        }
        Ok(Token {
            kind: TokenKind::String(String::from_utf8(value)?),
            span: Span {
                start,
                end: self.position,
            },
        })
    }

    fn hex_escape(&mut self, n: usize, span: Span) -> Result<Vec<u8>, Error> {
        let hex = self
            .input
            .get(self.position..self.position + n)
            .ok_or(Error::InvalidEscape { span })?;
        let code = u32::from_str_radix(hex, 16).map_err(|_| Error::InvalidEscape { span })?;
        let ch = char::from_u32(code).ok_or(Error::InvalidEscape { span })?;
        let value = ch.encode_utf8(&mut [0; 4]).as_bytes().to_vec();
        self.position += n;
        Ok(value)
    }

    fn is_identifier_start(&self, byte: u8) -> bool {
        byte.is_ascii_alphabetic() || byte == b'_' || byte >= 0x80
    }

    fn is_identifier_continue(&self, byte: u8) -> bool {
        self.is_identifier_start(byte) || byte.is_ascii_digit()
    }

    fn is_quote(&self, byte: u8) -> bool {
        matches!(byte, b'"' | b'\'')
    }

    fn is_whitespace(&self, byte: u8) -> bool {
        matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
    }

    fn maybe_handle_comment(&mut self, byte: u8) -> Result<bool, Error> {
        match (byte, self.peek_at(1)) {
            (b'/', Some(b'/')) => {
                self.skip_line_comment();
                Ok(true)
            }
            (b'/', Some(b'*')) => {
                self.skip_block_comment()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn maybe_handle_operator_or_punct(&mut self, byte1: u8, start: usize) -> Option<Token> {
        if let Some(byte2) = self.peek_at(1) {
            match (byte1, byte2) {
                (b'+', b'=') => {
                    self.position += 2;
                    Some(Token {
                        kind: TokenKind::Op(Op::PlusEq),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    })
                }
                (b'=', b'~') => {
                    self.position += 2;
                    Some(Token {
                        kind: TokenKind::Op(Op::EqTilde),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    })
                }
                (b'.', b'.') => {
                    self.position += 2;
                    Some(Token {
                        kind: TokenKind::Punct(Punct::DotDot),
                        span: Span {
                            start,
                            end: self.position,
                        },
                    })
                }
                _ => {
                    if let Ok(text) = std::str::from_utf8(&[byte1, byte2])
                        && let Ok(value) = Op::from_str(text)
                    {
                        self.position += 2;
                        Some(Token {
                            kind: TokenKind::Op(value),
                            span: Span {
                                start,
                                end: self.position,
                            },
                        })
                    } else if let Ok(text) = std::str::from_utf8(&[byte1])
                        && let Ok(value) = Op::from_str(text)
                    {
                        self.position += 1;
                        Some(Token {
                            kind: TokenKind::Op(value),
                            span: Span {
                                start,
                                end: self.position,
                            },
                        })
                    } else if let Ok(value) = Punct::try_from(byte1) {
                        self.position += 1;
                        Some(Token {
                            kind: TokenKind::Punct(value),
                            span: Span {
                                start,
                                end: self.position,
                            },
                        })
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    fn peek(&self) -> Option<u8> {
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        let index = self.position.checked_add(offset)?;
        self.input.as_bytes().get(index).copied()
    }

    fn skip_block_comment(&mut self) -> Result<(), Error> {
        let start = self.position;
        let mut closed = false;
        self.position += 2;
        while let Some(byte) = self.peek() {
            if matches!(byte, b'*') && self.peek_at(1) == Some(b'/') {
                self.position += 2;
                closed = true;
                break;
            }
            self.position += 1;
        }
        match closed {
            true => Ok(()),
            false => Err(Error::UnterminatedComment {
                span: Span {
                    start,
                    end: self.position,
                },
            }),
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
    fn test_lexer_case_one() {
        let mut lexer = Lexer::new(
            "MATCH (p:Person {age: 30, gender: $gender, income > 123.45})-[a:Attends]->(s:School)
                WHERE p.fname = \"Ralph\"
                  AND p.mname = \"\\\"Ralphie\\\"\"
                  AND p.lname = \"Wiggum\"
                  AND p.`is a` = \"Fire Engine\"
                  AND p.emoji = \"\\U0001F600\"
                RETURN p.name, a, s, count(p.income) + 1"
                .to_string(),
        );
        let Ok(tokens) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert_eq!(tokens.len(), 80);
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
        assert_eq!(tokens[19].kind, TokenKind::Op(Op::Minus));
        assert_eq!(tokens[20].kind, TokenKind::Punct(Punct::LBracket));
        assert_eq!(tokens[21].kind, TokenKind::Identifier("a".to_string()));
        assert_eq!(tokens[22].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(
            tokens[23].kind,
            TokenKind::Identifier("Attends".to_string())
        );
        assert_eq!(tokens[24].kind, TokenKind::Punct(Punct::RBracket));
        assert_eq!(tokens[25].kind, TokenKind::Op(Op::Minus));
        assert_eq!(tokens[26].kind, TokenKind::Op(Op::Gt));
        assert_eq!(tokens[27].kind, TokenKind::Punct(Punct::LParen));
        assert_eq!(tokens[28].kind, TokenKind::Identifier("s".to_string()));
        assert_eq!(tokens[29].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[30].kind, TokenKind::Identifier("School".to_string()));
        assert_eq!(tokens[31].kind, TokenKind::Punct(Punct::RParen));
        assert_eq!(tokens[32].kind, TokenKind::Keyword(Keyword::Where));
        assert_eq!(tokens[33].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[34].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[35].kind, TokenKind::Identifier("fname".to_string()));
        assert_eq!(tokens[36].kind, TokenKind::Op(Op::Eq));
        assert_eq!(tokens[37].kind, TokenKind::String("Ralph".to_string()));
        assert_eq!(tokens[38].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[39].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[40].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[41].kind, TokenKind::Identifier("mname".to_string()));
        assert_eq!(tokens[42].kind, TokenKind::Op(Op::Eq));
        assert_eq!(
            tokens[43].kind,
            TokenKind::String("\"Ralphie\"".to_string())
        );
        assert_eq!(tokens[44].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[45].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[46].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[47].kind, TokenKind::Identifier("lname".to_string()));
        assert_eq!(tokens[48].kind, TokenKind::Op(Op::Eq));
        assert_eq!(tokens[49].kind, TokenKind::String("Wiggum".to_string()));
        assert_eq!(tokens[50].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[51].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[52].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[53].kind, TokenKind::Identifier("is a".to_string()));
        assert_eq!(tokens[54].kind, TokenKind::Op(Op::Eq));
        assert_eq!(
            tokens[55].kind,
            TokenKind::String("Fire Engine".to_string())
        );
        assert_eq!(tokens[56].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[57].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[58].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[59].kind, TokenKind::Identifier("emoji".to_string()));
        assert_eq!(tokens[60].kind, TokenKind::Op(Op::Eq));
        assert_eq!(tokens[61].kind, TokenKind::String('😀'.to_string()));
        assert_eq!(tokens[62].kind, TokenKind::Keyword(Keyword::Return));
        assert_eq!(tokens[63].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[64].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[65].kind, TokenKind::Identifier("name".to_string()));
        assert_eq!(tokens[66].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[67].kind, TokenKind::Identifier("a".to_string()));
        assert_eq!(tokens[68].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[69].kind, TokenKind::Identifier("s".to_string()));
        assert_eq!(tokens[70].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[71].kind, TokenKind::Identifier("count".to_string()));
        assert_eq!(tokens[72].kind, TokenKind::Punct(Punct::LParen));
        assert_eq!(tokens[73].kind, TokenKind::Identifier("p".to_string()));
        assert_eq!(tokens[74].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[75].kind, TokenKind::Identifier("income".to_string()));
        assert_eq!(tokens[76].kind, TokenKind::Punct(Punct::RParen));
        assert_eq!(tokens[77].kind, TokenKind::Op(Op::Plus));
        assert_eq!(tokens[78].kind, TokenKind::Integer(1));
        assert_eq!(tokens[79].kind, TokenKind::Eof);
    }

    #[test]
    fn test_lexer_case_two() {
        let mut lexer = Lexer::new(
            "MATCH (a:Person)-[:KNOWS*1..3]->(b:Person)
             /* This is a comment that should be discarded */
             WHERE b.foo > 1e3
               AND b.bar <> .005
               AND b.baz < -5
               AND b.qux = 1e-5
             RETURN b, \"caf\\u00E9\" AS word, c-1"
                .to_string(),
        );
        let Ok(tokens) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        //assert_eq!(tokens.len(), 58);
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Match));
        assert_eq!(tokens[1].kind, TokenKind::Punct(Punct::LParen));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("a".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[4].kind, TokenKind::Identifier("Person".to_string()));
        assert_eq!(tokens[5].kind, TokenKind::Punct(Punct::RParen));
        assert_eq!(tokens[6].kind, TokenKind::Op(Op::Minus));
        assert_eq!(tokens[7].kind, TokenKind::Punct(Punct::LBracket));
        assert_eq!(tokens[8].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[9].kind, TokenKind::Identifier("KNOWS".to_string()));
        assert_eq!(tokens[10].kind, TokenKind::Op(Op::Star));
        assert_eq!(tokens[11].kind, TokenKind::Integer(1));
        assert_eq!(tokens[12].kind, TokenKind::Punct(Punct::DotDot));
        assert_eq!(tokens[13].kind, TokenKind::Integer(3));
        assert_eq!(tokens[14].kind, TokenKind::Punct(Punct::RBracket));
        assert_eq!(tokens[15].kind, TokenKind::Op(Op::Minus));
        assert_eq!(tokens[16].kind, TokenKind::Op(Op::Gt));
        assert_eq!(tokens[17].kind, TokenKind::Punct(Punct::LParen));
        assert_eq!(tokens[18].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[19].kind, TokenKind::Punct(Punct::Colon));
        assert_eq!(tokens[20].kind, TokenKind::Identifier("Person".to_string()));
        assert_eq!(tokens[21].kind, TokenKind::Punct(Punct::RParen));
        assert_eq!(tokens[22].kind, TokenKind::Keyword(Keyword::Where));
        assert_eq!(tokens[23].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[24].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[25].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[26].kind, TokenKind::Op(Op::Gt));
        assert_eq!(tokens[27].kind, TokenKind::Float(1000.0));
        assert_eq!(tokens[28].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[29].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[30].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[31].kind, TokenKind::Identifier("bar".to_string()));
        assert_eq!(tokens[32].kind, TokenKind::Op(Op::Ne));
        assert_eq!(tokens[33].kind, TokenKind::Float(0.005));
        assert_eq!(tokens[34].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[35].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[36].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[37].kind, TokenKind::Identifier("baz".to_string()));
        assert_eq!(tokens[38].kind, TokenKind::Op(Op::Lt));
        assert_eq!(tokens[39].kind, TokenKind::Op(Op::Minus));
        assert_eq!(tokens[40].kind, TokenKind::Integer(5));
        assert_eq!(tokens[41].kind, TokenKind::Keyword(Keyword::And));
        assert_eq!(tokens[42].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[43].kind, TokenKind::Punct(Punct::Dot));
        assert_eq!(tokens[44].kind, TokenKind::Identifier("qux".to_string()));
        assert_eq!(tokens[45].kind, TokenKind::Op(Op::Eq));
        assert_eq!(tokens[46].kind, TokenKind::Float(0.00001));
        assert_eq!(tokens[47].kind, TokenKind::Keyword(Keyword::Return));
        assert_eq!(tokens[48].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[49].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[50].kind, TokenKind::String("café".to_string()));
        assert_eq!(tokens[51].kind, TokenKind::Keyword(Keyword::As));
        assert_eq!(tokens[52].kind, TokenKind::Identifier("word".to_string()));
        assert_eq!(tokens[53].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[54].kind, TokenKind::Identifier("c".to_string()));
        assert_eq!(tokens[55].kind, TokenKind::Op(Op::Minus));
        assert_eq!(tokens[56].kind, TokenKind::Integer(1));
        assert_eq!(tokens[57].kind, TokenKind::Eof);
    }

    #[test]
    fn test_lexer_case_three() {
        let mut lexer = Lexer::new("RETURN \"café\"".to_string());
        let Ok(tokens) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Return));
        assert_eq!(tokens[1].kind, TokenKind::String("café".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn test_lexer_case_four() {
        let mut lexer = Lexer::new("RETURN 99999999999999999999".to_string());
        let Err(err) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert!(matches!(err, Error::ParseError { .. }));
    }

    #[test]
    fn test_lexer_case_five() {
        let mut lexer = Lexer::new("RETURN 1e400".to_string());
        let Err(err) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert!(matches!(err, Error::NumberOutOfRange { .. }));
    }

    #[test]
    fn test_lexer_case_six() {
        let mut lexer = Lexer::new("RETURN $ + 1".to_string());
        let Err(err) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert!(matches!(err, Error::InvalidParameter { .. }));
    }

    #[test]
    fn test_lexer_case_seven() {
        let mut lexer = Lexer::new("RETURN `` AS c".to_string());
        let Err(err) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert!(matches!(err, Error::InvalidIdentifier { .. }));
    }

    #[test]
    fn test_lexer_case_eight() {
        let mut lexer = Lexer::new("RETURN /* start of unfinished comment".to_string());
        let Err(err) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert!(matches!(err, Error::UnterminatedComment { .. }));
    }

    #[test]
    fn test_lexer_case_nine() {
        let mut lexer = Lexer::new("RETURN 0xff, 0o755, 0O755".to_string());
        let Ok(tokens) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Return));
        assert_eq!(tokens[1].kind, TokenKind::Integer(255));
        assert_eq!(tokens[2].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[3].kind, TokenKind::Integer(493));
        assert_eq!(tokens[4].kind, TokenKind::Punct(Punct::Comma));
        assert_eq!(tokens[5].kind, TokenKind::Integer(493));
        assert_eq!(tokens[6].kind, TokenKind::Eof);
    }

    #[test]
    fn test_lexer_case_ten() {
        let mut lexer = Lexer::new("RETURN 0x56BC75E2D630FFFFF".to_string());
        let Err(err) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert!(matches!(err, Error::ParseError { .. }));
    }

    #[test]
    fn test_lexer_case_eleven() {
        let mut lexer = Lexer::new("RETURN 0o2000000000000000000000".to_string());
        let Err(err) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        assert!(matches!(err, Error::ParseError { .. }));
    }

    #[test]
    fn test_lexer_case_twelve() {
        let mut lexer = Lexer::new("RETURN 1e+".to_string());
        let Err(err) = lexer.lex() else {
            panic!("{:?}", lexer.lex().err());
        };
        let Error::ParseFloatError { span, .. } = err else {
            panic!("wrong error: {err:?}");
        };
        assert_eq!(span, Span { start: 7, end: 10 });
    }
}
