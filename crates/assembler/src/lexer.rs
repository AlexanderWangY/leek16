use crate::{
    error::LexError,
    token::{RegisterKind, Token, TokenKind},
};

pub struct Lexer<'a> {
    source: &'a [u8],
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(t) = self.next_token()? {
            tokens.push(t);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_whitespace();

        let c = match self.current() {
            Some(c) => c,
            None => return Ok(None),
        };

        let result = match c {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_word(),
            b'0'..=b'9' => self.lex_number(),
            b'>' => self.lex_greater(),

            // This can also be the start of an arrow btw
            b'<' => self.lex_less(),
            b'=' => self.lex_equal(),
            b':' => self.lex_colon(),
            b'[' => self.lex_lbracket(),
            b']' => self.lex_rbracket(),
            b'-' => self.lex_hyphen(),
            b'+' => self.lex_plus(),
            b'!' => self.lex_bang(),
            b'&' => self.lex_ampersand(),
            b'|' => self.lex_pipe(),
            _ => {
                return Err(LexError::UnexpectedCharacter {
                    ch: c,
                    line: self.line,
                    column: self.column,
                });
            }
        };

        result.map(Some)
    }

    fn lex_word(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        while let Some(c) = self.current() {
            match c {
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => {
                    self.advance();
                }
                _ => break,
            }
        }

        let word = str::from_utf8(&self.source[start..self.pos]).unwrap();

        let kind = match word {
            "JMP" => TokenKind::Jmp,
            "BRT" => TokenKind::Brt,
            "BRF" => TokenKind::Brf,
            "FUNC" => TokenKind::Func,
            "END" => TokenKind::FuncEnd,
            "R0" => TokenKind::Register(RegisterKind::R0),
            "R1" => TokenKind::Register(RegisterKind::R1),
            "R2" => TokenKind::Register(RegisterKind::R2),
            "R3" => TokenKind::Register(RegisterKind::R3),
            "R4" => TokenKind::Register(RegisterKind::R4),
            "R5" => TokenKind::Register(RegisterKind::R5),
            "R6" => TokenKind::Register(RegisterKind::R6),
            "R7" => TokenKind::Register(RegisterKind::R7),
            "SP" => TokenKind::Register(RegisterKind::Sp),
            "PC" => TokenKind::Register(RegisterKind::Pc),

            _ => TokenKind::Identifier(word.to_string()),
        };

        Ok(Token {
            kind,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        while let Some(c) = self.current() {
            match c {
                b'0'..=b'9' => self.advance(),
                _ => break,
            }
        }

        let num: u16 = str::from_utf8(&self.source[start..self.pos])
            .unwrap()
            .parse()
            .unwrap();

        Ok(Token {
            kind: TokenKind::Number(num),
            line: start_line,
            column: start_col,
        })
    }

    fn lex_greater(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a >, so lets move on
        self.advance();

        let kind = match self.current() {
            Some(b'>') => {
                self.advance();
                TokenKind::ShiftRight
            }
            Some(b'=') => {
                self.advance();
                TokenKind::GreaterEq
            }
            _ => TokenKind::Greater,
        };

        Ok(Token {
            kind,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_less(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // Consume first <
        self.advance();
        let kind = match self.current() {
            Some(b'<') => {
                self.advance();
                TokenKind::ShiftLeft
            }
            Some(b'-') => {
                self.advance();
                TokenKind::ArrowLeft
            }
            Some(b'=') => {
                self.advance();
                TokenKind::LessEq
            }
            _ => TokenKind::Less,
        };

        Ok(Token {
            kind,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_equal(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a =, so move on
        self.advance();

        let kind = match self.current() {
            Some(b'=') => {
                self.advance();
                TokenKind::Equal
            }
            _ => TokenKind::Greater,
        };

        Ok(Token {
            kind,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_colon(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a :, so move on
        self.advance();

        let kind = match self.current() {
            Some(b'=') => {
                self.advance();
                TokenKind::Assign
            }
            _ => TokenKind::Colon,
        };

        Ok(Token {
            kind,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_lbracket(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a [, so move on
        self.advance();

        Ok(Token {
            kind: TokenKind::LBracket,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_rbracket(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a ], so move on
        self.advance();

        Ok(Token {
            kind: TokenKind::RBracket,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_hyphen(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a -, so move on
        self.advance();

        let kind = match self.current() {
            Some(b'>') => {
                self.advance();
                TokenKind::ArrowRight
            }
            _ => TokenKind::Minus,
        };

        Ok(Token {
            kind,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_plus(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a +, so move on
        self.advance();

        Ok(Token {
            kind: TokenKind::Plus,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_bang(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a !, so move on
        self.advance();

        let kind = match self.current() {
            Some(b'=') => {
                self.advance();
                TokenKind::NotEqual
            }
            _ => TokenKind::Not,
        };

        Ok(Token {
            kind,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_ampersand(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a &, so move on
        self.advance();
        Ok(Token {
            kind: TokenKind::And,
            line: start_line,
            column: start_col,
        })
    }

    fn lex_pipe(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;

        // At this point we already now pos is a |, so move on
        self.advance();
        Ok(Token {
            kind: TokenKind::Or,
            line: start_line,
            column: start_col,
        })
    }

    fn current(&self) -> Option<u8> {
        self.source.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            match c {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) {
        if let Some(c) = self.current() {
            self.pos += 1;

            if c == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }
}
