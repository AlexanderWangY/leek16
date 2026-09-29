use crate::{
    ast::{
        Address, Ast, BinaryOp, CompareOp, Function, Instruction, Item, Label, Operand, UnaryOp,
    },
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

    pub fn parse(&mut self) -> Result<Ast, ParseError> {
        let mut ast = Ast::new();

        while !self.is_eof() {
            let token = self.current().ok_or(ParseError::MissingToken)?;

            let item = match token.kind {
                TokenKind::Func => Item::Function(self.parse_function()?),
                _ => Item::Instruction(self.parse_instruction()?),
            };

            ast.items.push(item);
        }

        Ok(ast)
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
                TokenKind::Register(_) | TokenKind::Number(_) => {
                    return self.parse_operand_instruction();
                }
                TokenKind::LBracket => return self.parse_memory_transfer(),
                TokenKind::Jmp | TokenKind::Brt | TokenKind::Brf => return self.parse_branch(),
                _ => return Err(ParseError::UnexpectedToken),
            }
        }

        Err(ParseError::MissingToken)
    }

    fn parse_operand_instruction(&mut self) -> Result<Instruction, ParseError> {
        // If next is := , that is assignment, if it is >, <, >=, <= or == or !=, it is binary op
        let token = self.peek(1).ok_or(ParseError::MissingToken)?;

        match &token.kind {
            TokenKind::Assign => self.parse_assign(),
            TokenKind::Less
            | TokenKind::LessEq
            | TokenKind::Greater
            | TokenKind::GreaterEq
            | TokenKind::Equal
            | TokenKind::NotEqual => self.parse_compare(),
            TokenKind::ArrowLeft | TokenKind::ArrowRight => self.parse_memory_transfer(),
            _ => Err(ParseError::UnexpectedToken),
        }
    }
    fn parse_memory_transfer(&mut self) -> Result<Instruction, ParseError> {
        // If first is a register, then this is either R1 -> [R0] or R1 <- [R0]
        // If first is a [, then this is either [R0] -> R1 or [R0] <- R1

        let token = self.current().ok_or(ParseError::MissingToken)?;

        match token.kind {
            TokenKind::LBracket => {
                let addr = self.parse_address()?;
                let arrow = self.current().ok_or(ParseError::MissingToken)?.kind.clone();
                self.advance();

                let reg = self.expect_register()?;
                self.advance();

                match arrow {
                    TokenKind::ArrowRight => Ok(Instruction::Load { dst: reg, addr }),
                    TokenKind::ArrowLeft => Ok(Instruction::Store {
                        addr,
                        src: Operand::Register(reg),
                    }),
                    _ => Err(ParseError::UnexpectedToken),
                }
            }
            TokenKind::Register(_) => {
                let reg = self.expect_register()?;
                self.advance();

                let arrow = self.current().ok_or(ParseError::MissingToken)?.kind.clone();
                self.advance();

                let addr = self.parse_address()?;

                match arrow {
                    TokenKind::ArrowRight => Ok(Instruction::Store {
                        addr,
                        src: Operand::Register(reg),
                    }),
                    TokenKind::ArrowLeft => Ok(Instruction::Load { dst: reg, addr }),
                    _ => Err(ParseError::UnexpectedToken),
                }
            }
            TokenKind::Number(_) => todo!(),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    fn parse_address(&mut self) -> Result<Address, ParseError> {
        self.expect(TokenKind::LBracket)?;
        self.advance();
        let addr = match self
            .current()
            .ok_or(ParseError::UnexpectedToken)?
            .kind
            .clone()
        {
            TokenKind::Register(reg) => Address::Register(reg),
            TokenKind::Number(num) => Address::Immediate(num),
            _ => return Err(ParseError::UnexpectedToken),
        };
        self.advance();
        self.expect(TokenKind::RBracket)?;
        self.advance();
        Ok(addr)
    }

    fn parse_branch(&mut self) -> Result<Instruction, ParseError> {
        let kind = self.current().ok_or(ParseError::MissingToken)?.kind.clone();

        self.advance();

        let label = self.expect_identifier()?;
        self.advance();

        match kind {
            TokenKind::Jmp => Ok(Instruction::Jump {
                target: Label(label),
            }),
            TokenKind::Brt => Ok(Instruction::BranchTrue {
                target: Label(label),
            }),
            TokenKind::Brf => Ok(Instruction::BranchFalse {
                target: Label(label),
            }),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    fn parse_compare(&mut self) -> Result<Instruction, ParseError> {
        let lhs = match self.current().ok_or(ParseError::MissingToken)?.kind.clone() {
            TokenKind::Register(reg) => Operand::Register(reg),
            TokenKind::Number(num) => Operand::Immediate(num),
            _ => return Err(ParseError::UnexpectedToken),
        };
        self.advance();

        let op = match self.current().ok_or(ParseError::MissingToken)?.kind {
            TokenKind::Greater => CompareOp::Greater,
            TokenKind::GreaterEq => CompareOp::GreaterEq,
            TokenKind::Less => CompareOp::Less,
            TokenKind::LessEq => CompareOp::LessEq,
            TokenKind::Equal => CompareOp::Equal,
            TokenKind::NotEqual => CompareOp::NotEqual,
            _ => return Err(ParseError::UnexpectedToken),
        };
        self.advance();

        let rhs = match self.current().ok_or(ParseError::MissingToken)?.kind.clone() {
            TokenKind::Register(reg) => Operand::Register(reg),
            TokenKind::Number(num) => Operand::Immediate(num),
            _ => return Err(ParseError::UnexpectedToken),
        };
        self.advance();

        if matches!(lhs, Operand::Immediate(_)) && matches!(rhs, Operand::Immediate(_)) {
            return Err(ParseError::UnexpectedToken);
        }

        Ok(Instruction::Compare { op, lhs, rhs })
    }

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
        self.expect(TokenKind::Assign)?;
        self.advance(); // Skip :=

        // Check for unary !
        if matches!(
            &self.current().ok_or(ParseError::MissingToken)?.kind,
            TokenKind::Not
        ) {
            self.advance();
            let src = self.expect_register()?;
            self.advance();

            return Ok(Instruction::Unary {
                dst: dest_reg,
                op: UnaryOp::Not,
                src,
            });
        }

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

            Ok(Instruction::Binary {
                dst: dest_reg,
                op,
                lhs,
                rhs,
            })
        } else {
            let slot0 = self.current().ok_or(ParseError::MissingToken)?;
            let lhs = match &slot0.kind {
                TokenKind::Register(kind) => Operand::Register(kind.clone()),
                TokenKind::Number(num) => Operand::Immediate(*num),
                _ => return Err(ParseError::UnexpectedToken),
            };
            self.advance();

            Ok(Instruction::Move {
                dst: dest_reg,
                src: lhs,
            })
        }
    }

    fn expect_identifier(&self) -> Result<String, ParseError> {
        let token = self.tokens.get(self.pos).ok_or(ParseError::MissingToken)?;

        match &token.kind {
            TokenKind::Identifier(label) => Ok(label.clone()),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    fn expect_register(&self) -> Result<RegisterKind, ParseError> {
        let token = self.tokens.get(self.pos).ok_or(ParseError::MissingToken)?;

        match &token.kind {
            TokenKind::Register(kind) => Ok(kind.clone()),
            _ => Err(ParseError::UnexpectedToken),
        }
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
