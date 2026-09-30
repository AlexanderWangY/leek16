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
            let token = self.current_or("function or instruction")?;

            let item = match token.kind {
                TokenKind::Func => Item::Function(self.parse_function()?),
                TokenKind::FuncEnd => {
                    return Err(ParseError::UnmatchedFunctionEnd {
                        line: token.line,
                        column: token.column,
                    });
                }
                _ => Item::Instruction(self.parse_instruction()?),
            };

            ast.items.push(item);
        }

        Ok(ast)
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let start = self.current_or("`FUNC`")?;
        self.advance();
        let mut instructions = Vec::new();

        let label = Label(self.expect_identifier()?);
        self.advance();

        self.expect(TokenKind::Colon, "`:`")?;
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
                TokenKind::Func => {
                    return Err(ParseError::NestedFunction {
                        outer: label.0,
                        line: token.line,
                        column: token.column,
                    });
                }
                _ => {
                    instructions.push(self.parse_instruction()?);
                }
            }
        }

        Err(ParseError::MissingFunctionEnd {
            name: label.0,
            line: start.line,
            column: start.column,
        })
    }

    fn parse_instruction(&mut self) -> Result<Instruction, ParseError> {
        if let Some(token) = self.current() {
            match &token.kind {
                TokenKind::Register(_) | TokenKind::Number(_) => {
                    return self.parse_operand_instruction();
                }
                TokenKind::LBracket => return self.parse_memory_transfer(),
                TokenKind::Jmp | TokenKind::Brt | TokenKind::Brf => return self.parse_branch(),
                _ => return Err(Self::unexpected(token, "instruction")),
            }
        }

        Err(self.missing("instruction"))
    }

    fn parse_operand_instruction(&mut self) -> Result<Instruction, ParseError> {
        // If next is := , that is assignment, if it is >, <, >=, <= or == or !=, it is binary op
        let expected = "`:=`, comparison operator, `->` or `<-`";
        let token = self.peek(1).ok_or_else(|| self.missing(expected))?;

        match &token.kind {
            TokenKind::Assign => self.parse_assign(),
            TokenKind::Less
            | TokenKind::LessEq
            | TokenKind::Greater
            | TokenKind::GreaterEq
            | TokenKind::Equal
            | TokenKind::NotEqual => self.parse_compare(),
            TokenKind::ArrowLeft | TokenKind::ArrowRight => self.parse_memory_transfer(),
            _ => Err(Self::unexpected(token, expected)),
        }
    }
    fn parse_memory_transfer(&mut self) -> Result<Instruction, ParseError> {
        // If first is a register, then this is either R1 -> [R0] or R1 <- [R0]
        // If first is a [, then this is either [R0] -> R1 or [R0] <- R1

        let token = self.current_or("register, number or `[`")?;

        match token.kind {
            TokenKind::LBracket => {
                let addr = self.parse_address()?;
                let arrow = self.expect_arrow()?;
                self.advance();

                let operand = self.current_or("register or number")?;
                let src = self.expect_operand()?;
                self.advance();

                match (arrow, src) {
                    (TokenKind::ArrowRight, Operand::Register(reg)) => {
                        Ok(Instruction::Load { dst: reg, addr })
                    }
                    (TokenKind::ArrowRight, Operand::Immediate(value)) => {
                        Err(ParseError::LoadIntoImmediate {
                            value,
                            line: operand.line,
                            column: operand.column,
                        })
                    }
                    (TokenKind::ArrowLeft, src) => Self::store(addr, src, token),
                    _ => unreachable!(),
                }
            }
            TokenKind::Register(_) => {
                let reg = self.expect_register()?;
                self.advance();

                let arrow = self.expect_arrow()?;
                self.advance();

                let addr = self.parse_address()?;

                match arrow {
                    TokenKind::ArrowRight => Ok(Instruction::Store {
                        addr,
                        src: Operand::Register(reg),
                    }),
                    TokenKind::ArrowLeft => Ok(Instruction::Load { dst: reg, addr }),
                    _ => unreachable!(),
                }
            }
            TokenKind::Number(value) => {
                self.advance();

                let arrow = self.expect_arrow()?;
                self.advance();

                let addr = self.parse_address()?;

                match arrow {
                    TokenKind::ArrowRight => Self::store(addr, Operand::Immediate(value), token),
                    TokenKind::ArrowLeft => Err(ParseError::LoadIntoImmediate {
                        value,
                        line: token.line,
                        column: token.column,
                    }),
                    _ => unreachable!(),
                }
            }
            _ => Err(Self::unexpected(token, "register, number or `[`")),
        }
    }

    fn store(addr: Address, src: Operand, token: &Token) -> Result<Instruction, ParseError> {
        if let (&Address::Immediate(address), &Operand::Immediate(value)) = (&addr, &src) {
            return Err(ParseError::StoreImmediateToImmediate {
                value,
                addr: address,
                line: token.line,
                column: token.column,
            });
        }

        Ok(Instruction::Store { addr, src })
    }

    fn parse_address(&mut self) -> Result<Address, ParseError> {
        self.expect(TokenKind::LBracket, "`[`")?;
        self.advance();
        let token = self.current_or("register or number")?;
        let addr = match &token.kind {
            TokenKind::Register(reg) => Address::Register(reg.clone()),
            TokenKind::Number(num) => Address::Immediate(*num),
            _ => return Err(Self::unexpected(token, "register or number")),
        };
        self.advance();
        self.expect(TokenKind::RBracket, "`]`")?;
        self.advance();
        Ok(addr)
    }

    fn parse_branch(&mut self) -> Result<Instruction, ParseError> {
        let token = self.current_or("branch")?;

        self.advance();

        let label = self.expect_identifier()?;
        self.advance();

        match token.kind {
            TokenKind::Jmp => Ok(Instruction::Jump {
                target: Label(label),
            }),
            TokenKind::Brt => Ok(Instruction::BranchTrue {
                target: Label(label),
            }),
            TokenKind::Brf => Ok(Instruction::BranchFalse {
                target: Label(label),
            }),
            _ => Err(Self::unexpected(token, "branch")),
        }
    }

    fn parse_compare(&mut self) -> Result<Instruction, ParseError> {
        let start = self.current_or("register or number")?;
        let lhs = self.expect_operand()?;
        self.advance();

        let token = self.current_or("comparison operator")?;
        let op = match token.kind {
            TokenKind::Greater => CompareOp::Greater,
            TokenKind::GreaterEq => CompareOp::GreaterEq,
            TokenKind::Less => CompareOp::Less,
            TokenKind::LessEq => CompareOp::LessEq,
            TokenKind::Equal => CompareOp::Equal,
            TokenKind::NotEqual => CompareOp::NotEqual,
            _ => return Err(Self::unexpected(token, "comparison operator")),
        };
        self.advance();

        let rhs = self.expect_operand()?;
        self.advance();

        if let (&Operand::Immediate(lhs), &Operand::Immediate(rhs)) = (&lhs, &rhs) {
            return Err(ParseError::ImmediateCompare {
                lhs,
                rhs,
                line: start.line,
                column: start.column,
            });
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
        self.expect(TokenKind::Assign, "`:=`")?;
        self.advance(); // Skip :=

        // Check for unary !
        if matches!(
            &self.current_or("register, number or `!`")?.kind,
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
            let lhs = self.expect_operand()?;

            self.advance();
            self.advance();

            let rhs = self.expect_operand()?;
            self.advance();

            Ok(Instruction::Binary {
                dst: dest_reg,
                op,
                lhs,
                rhs,
            })
        } else {
            let lhs = self.expect_operand()?;
            self.advance();

            Ok(Instruction::Move {
                dst: dest_reg,
                src: lhs,
            })
        }
    }

    fn expect_identifier(&self) -> Result<String, ParseError> {
        let token = self.current_or("identifier")?;

        match &token.kind {
            TokenKind::Identifier(label) => Ok(label.clone()),
            _ => Err(Self::unexpected(token, "identifier")),
        }
    }

    fn expect_register(&self) -> Result<RegisterKind, ParseError> {
        let token = self.current_or("register")?;

        match &token.kind {
            TokenKind::Register(kind) => Ok(kind.clone()),
            _ => Err(Self::unexpected(token, "register")),
        }
    }

    fn expect_operand(&self) -> Result<Operand, ParseError> {
        let token = self.current_or("register or number")?;

        match &token.kind {
            TokenKind::Register(kind) => Ok(Operand::Register(kind.clone())),
            TokenKind::Number(num) => Ok(Operand::Immediate(*num)),
            _ => Err(Self::unexpected(token, "register or number")),
        }
    }

    fn expect_arrow(&self) -> Result<TokenKind, ParseError> {
        let token = self.current_or("`->` or `<-`")?;

        match &token.kind {
            TokenKind::ArrowLeft | TokenKind::ArrowRight => Ok(token.kind.clone()),
            _ => Err(Self::unexpected(token, "`->` or `<-`")),
        }
    }

    fn expect(&self, kind: TokenKind, expected: &'static str) -> Result<&'a Token, ParseError> {
        let token = self.current_or(expected)?;

        if token.kind != kind {
            return Err(Self::unexpected(token, expected));
        }

        Ok(token)
    }

    fn current_or(&self, expected: &'static str) -> Result<&'a Token, ParseError> {
        self.current().ok_or_else(|| self.missing(expected))
    }

    fn missing(&self, expected: &'static str) -> ParseError {
        let (line, column) = self
            .tokens
            .last()
            .map_or((1, 1), |token| (token.line, token.column));

        ParseError::MissingToken {
            expected,
            line,
            column,
        }
    }

    fn unexpected(token: &Token, expected: &'static str) -> ParseError {
        ParseError::UnexpectedToken {
            found: token.kind.clone(),
            expected,
            line: token.line,
            column: token.column,
        }
    }

    fn current(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn peek(&self, offset: usize) -> Option<&'a Token> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
