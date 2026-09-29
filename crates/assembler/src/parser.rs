use crate::{
    ast::{Ast, BinaryOp, Function, Instruction, Item, Label, Operand},
    error::ParseError,
    token::{RegisterKind, Token, TokenKind},
};

pub struct Parser<'a> {
    pos: usize,
    tokens: &'a [Token],
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { pos: 0, tokens }
    }

    pub fn parse(&self) -> Result<Ast, ParseError> {
        let mut ast = Ast::new();

        while !self.is_eof() {
            let token = self.current().ok_or_else(|| ParseError::MissingToken)?;

            let item = match token.kind {
                TokenKind::Func => Item::Function(self.parse_function()?),
                _ => Item::Instruction(self.parse_instruction()?),
            };
        }

        Ok(Ast {})
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        self.advance();
        let mut instructions = Vec::new();

        let label = Label(self.expect_identifier()?);
        self.advance();

        self.expect(TokenKind::Colon)?;
        self.advance();

        while let Some(token) = self.current() {
            match token.kind {
                TokenKind::FuncEnd => {
                    self.advance();

                    return Ok(Function {
                        label,
                        instructions,
                    });
                }
                _ => {
                    instructions.push(self.parse_instruction()?);
                }
            }
        }

        Err(ParseError::MissingFunctionEnd)
    }

    fn parse_instruction(&mut self) -> Result<Instruction, ParseError> {
        if let Some(token) = self.current() {
            match &token.kind {
                TokenKind::Register(_) => return self.parse_register_instruction(),
                TokenKind::LBracket => return self.parse_memory_transfer(),
                TokenKind::Jmp | TokenKind::Brt | TokenKind::Brf => return self.parse_branch(),
                _ => return Err(ParseError::UnexpectedToken),
            }
        }

        Err(ParseError::MissingToken)
    }

    fn parse_register_instruction(&mut self) -> Result<Instruction, ParseError> {
        // If next is := , that is assignment, if it is >, <, >=, <= or == or !=, it is binary op
        let token = self.peek(1).ok_or(ParseError::MissingToken)?;

        match &token.kind {
            TokenKind::Assign => {}
            TokenKind::Less => {}
        }
    }
    fn parse_memory_transfer(&mut self) -> Result<Instruction, ParseError> {}
    fn parse_branch(&mut self) -> Result<Instruction, ParseError> {}

    fn parse_assign(&mut self) -> Result<Instruction, ParseError> {
        // So basically at this point its R1 := xxxxx

        let op = self.peek(3).and_then(|token| match &token.kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::And => Some(BinaryOp::And),
            TokenKind::Or => Some(BinaryOp::Or),
            TokenKind::ShiftLeft => Some(BinaryOp::ShiftL),
            TokenKind::ShiftRight => Some(BinaryOp::ShiftR),
            _ => None,
        });

        let dest_reg = self.expect_register()?;
        self.advance(); // Skip R0
        self.advance(); // Skip := 

        if let Some(op) = op {
            // This means there was a valid operation after
            let slot0 = self.current().ok_or(ParseError::MissingToken)?;
            let lhs = match &slot0.kind {
                TokenKind::Register(kind) => Operand::Register(kind.clone()),
                TokenKind::Number(num) => Operand::Immediate(*num),
                _ => return Err(ParseError::UnexpectedToken),
            };

            self.advance();
            self.advance();

            let slot1 = self.current().ok_or(ParseError::MissingToken)?;
            let rhs = match &slot1.kind {
                TokenKind::Register(kind) => Operand::Register(kind.clone()),
                TokenKind::Number(num) => Operand::Immediate(*num),
                _ => return Err(ParseError::UnexpectedToken),
            };
            self.advance();

            return Ok(Instruction::Binary {
                dst: dest_reg,
                op,
                lhs,
                rhs,
            });
        } else {
            let slot0 = self.current().ok_or(ParseError::MissingToken)?;
            let lhs = match &slot0.kind {
                TokenKind::Register(kind) => Operand::Register(kind.clone()),
                TokenKind::Number(num) => Operand::Immediate(*num),
                _ => return Err(ParseError::UnexpectedToken),
            };
            self.advance();

            return Ok(Instruction::Move {
                dst: dest_reg,
                src: lhs,
            });
        }
    }

    fn expect_identifier(&self) -> Result<String, ParseError> {
        let token = self.tokens.get(self.pos).ok_or(ParseError::MissingToken)?;

        match &token.kind {
            TokenKind::Identifier(label) => return Ok(label.clone()),
            _ => return Err(ParseError::UnexpectedToken),
        };
    }

    fn expect_register(&self) -> Result<RegisterKind, ParseError> {
        let token = self.tokens.get(self.pos).ok_or(ParseError::MissingToken)?;

        match &token.kind {
            TokenKind::Register(kind) => return Ok(kind.clone()),
            _ => return Err(ParseError::UnexpectedToken),
        };
    }

    fn expect(&self, kind: TokenKind) -> Result<&'a Token, ParseError> {
        let token = self.tokens.get(self.pos).ok_or(ParseError::MissingToken)?;

        if token.kind != kind {
            return Err(ParseError::UnexpectedToken);
        }

        Ok(token)
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
