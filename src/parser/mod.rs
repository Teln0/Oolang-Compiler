use crate::lexer::{Token, TokenKind, BinOpTokenKind, DelimTokenKind, KeywordTokenKind};
use crate::reporting::TokenSpan;
use crate::ast::{ASTExpr, ASTOperator, ASTExprKind, ASTStatement, ASTStatementKind, ASTStatementBlock, ASTTypeInfo, ASTPath, ASTType, ASTRoot, ASTMod, ASTUse, ASTVisibility, ASTModifier, ASTGenericBound, ASTPartialTypeInfo, ASTMember, ASTTypeKind, ASTMemberKind, ASTNameAndType};
use crate::lexer::DelimTokenKind::SBracket;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OpPrecedence {
    Assignment,
    Or,
    And,
    Eq,
    Cmp,
    Term,
    Factor
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current_token: usize
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens,
            current_token: 0
        }
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.current_token];
        self.current_token += 1;
        t
    }

    fn advance_match(&mut self, expected: TokenKind) -> Result<(), ParserError> {
        let token = self.peek();
        self.advance();
        if token == expected {
            Ok(())
        }
        else {
            Err(ParserError::new(TokenSpan::new(self.current_token, 1), token))
        }
    }

    fn peek(&self) -> TokenKind {
        self.tokens[self.current_token].kind
    }

    fn statement_undo_ending(&self, statement: &mut ASTStatement) -> Result<(), ParserError> {
        match &statement.kind {
            ASTStatementKind::Expression(e) => {
                match &e.kind {
                    ASTExprKind::Block(_) => {
                        statement.ending = false;
                        return Ok(());
                    }
                    ASTExprKind::IfElse(_, _, _) => {
                        statement.ending = false;
                        return Ok(());
                    }
                    ASTExprKind::If(_, _) => {
                        statement.ending = false;
                        return Ok(());
                    }
                    ASTExprKind::While(_, _) => {
                        statement.ending = false;
                        return Ok(());
                    }
                    ASTExprKind::Loop(_) => {
                        statement.ending = false;
                        return Ok(());
                    }
                    ASTExprKind::Match() => {
                        statement.ending = false;
                        return Ok(());
                    }
                    ASTExprKind::For() => {
                        statement.ending = false;
                        return Ok(());
                    }
                    _ => {}
                }
            }
            _ => {}
        }


        let statement_end_loc = statement.span.base + statement.span.len;
        Err(ParserError::new(
            TokenSpan::new(statement_end_loc, 1),
            self.tokens[statement_end_loc].kind
        ))
    }

    fn parse_expression_primary(&mut self) -> Result<ASTExpr<'a>, ParserError> {
        let starting_token = self.current_token;
        let token = self.peek();
        match token {
            TokenKind::Keyword(KeywordTokenKind::While) => {
                self.advance();
                let cond = self.parse_expression()?;
                let block = self.parse_statement_block()?;
                Ok(ASTExpr {
                    kind: ASTExprKind::While(Box::new(cond), block),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                })
            }
            TokenKind::Keyword(KeywordTokenKind::If) => {
                self.advance();
                let cond = self.parse_expression()?;
                let block_if = self.parse_statement_block()?;
                if self.peek() == TokenKind::Keyword(KeywordTokenKind::Else) {
                    self.advance();
                    let block_else = self.parse_statement_block()?;
                    Ok(ASTExpr {
                        kind: ASTExprKind::IfElse(Box::new(cond), block_if, block_else),
                        span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                    })
                }
                else {
                    Ok(ASTExpr {
                        kind: ASTExprKind::If(Box::new(cond), block_if),
                        span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                    })
                }
            },
            TokenKind::Keyword(KeywordTokenKind::Loop) => {
                self.advance();
                let block = self.parse_statement_block()?;
                Ok(ASTExpr {
                    kind: ASTExprKind::Loop(block),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                })
            },
            TokenKind::OpeningDelim(DelimTokenKind::CBracket) => {
                let block = self.parse_statement_block()?;
                Ok(ASTExpr {
                    kind: ASTExprKind::Block(block),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                })
            },
            TokenKind::OpeningDelim(DelimTokenKind::Paren) => {
                self.advance();
                let inner = self.parse_expression()?;
                self.advance_match(TokenKind::ClosingDelim(DelimTokenKind::Paren))?;
                Ok(inner)
            },
            TokenKind::Null => {
                let res = Ok(ASTExpr {
                    kind: ASTExprKind::Null,
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                });
                self.advance();
                res
            },
            TokenKind::True => {
                let res = Ok(ASTExpr {
                    kind: ASTExprKind::Boolean(true),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                });
                self.advance();
                res
            },
            TokenKind::False => {
                let res = Ok(ASTExpr {
                    kind: ASTExprKind::Boolean(false),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                });
                self.advance();
                res
            },
            TokenKind::StringLiteral => {
                let res = Ok(ASTExpr {
                    kind: ASTExprKind::StringLiteral(self.tokens[self.current_token].string),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                });
                self.advance();
                res
            },
            TokenKind::Float => {
                let res = Ok(ASTExpr {
                    kind: ASTExprKind::Float(self.tokens[self.current_token].string),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                });
                self.advance();
                res
            },
            TokenKind::Num => {
                let res = Ok(ASTExpr {
                    kind: ASTExprKind::Num(self.tokens[self.current_token].string),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                });
                self.advance();
                res
            },
            TokenKind::Ident => {
                let res = Ok(ASTExpr {
                    kind: ASTExprKind::Ident(self.tokens[self.current_token].string),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                });
                self.advance();
                res
            },
            _ => {
                self.advance();
                Err(ParserError::new(TokenSpan::new_rn_ex(starting_token, self.current_token), token))
            }
        }
    }

    fn parse_expression_call(&mut self) -> Result<ASTExpr<'a>, ParserError> {
        let starting_token = self.current_token;

        let mut expr = self.parse_expression_primary()?;

        loop {
            expr = match self.peek() {
                TokenKind::Dot => {
                    self.advance();
                    let token_str = self.tokens[self.current_token].string;
                    self.advance_match(TokenKind::Ident)?;
                    ASTExpr {
                        kind: ASTExprKind::MemberAccess(Box::new(expr), token_str),
                        span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                    }
                },
                TokenKind::ColonColon => {
                    self.advance();
                    let token_str = self.tokens[self.current_token].string;
                    self.advance_match(TokenKind::Ident)?;
                    ASTExpr {
                        kind: ASTExprKind::StaticAccess(Box::new(expr), token_str),
                        span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                    }
                },
                TokenKind::OpeningDelim(DelimTokenKind::Paren) => {
                    self.advance();

                    if self.peek() == TokenKind::ClosingDelim(DelimTokenKind::Paren) {
                        ASTExpr {
                            kind: ASTExprKind::Call(Box::new(expr), vec![]),
                            span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                        }
                    }
                    else {
                        let mut args = vec![self.parse_expression()?];
                        while self.peek() == TokenKind::Comma {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        self.advance_match(TokenKind::ClosingDelim(DelimTokenKind::Paren))?;
                        ASTExpr {
                            kind: ASTExprKind::Call(Box::new(expr), args),
                            span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                        }
                    }
                },
                TokenKind::OpeningDelim(DelimTokenKind::SBracket) => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.advance_match(TokenKind::ClosingDelim(SBracket))?;
                    ASTExpr {
                        kind: ASTExprKind::Indexing(Box::new(expr), Box::new(index)),
                        span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                    }
                }
                _ => break
            }
        }

        Ok(expr)
    }

    fn parse_expression_prefix(&mut self) -> Result<ASTExpr<'a>, ParserError> {
        let starting_token = self.current_token;

        match self.peek() {
            TokenKind::PlusPlus => {
                self.advance();
                Ok(ASTExpr {
                    kind: ASTExprKind::PreOp(
                        ASTOperator::Inc,
                        Box::new(self.parse_expression_prefix()?)
                    ),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                })
            }
            TokenKind::MinusMinus => {
                self.advance();
                Ok(ASTExpr {
                    kind: ASTExprKind::PreOp(
                        ASTOperator::Dec,
                        Box::new(self.parse_expression_prefix()?)
                    ),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                })
            }
            TokenKind::Not => {
                self.advance();
                Ok(ASTExpr {
                    kind: ASTExprKind::PreOp(
                        ASTOperator::Not,
                        Box::new(self.parse_expression_prefix()?)
                    ),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                })
            }
            TokenKind::BinOp(BinOpTokenKind::Minus) => {
                self.advance();
                Ok(ASTExpr {
                    kind: ASTExprKind::PreOp(
                        ASTOperator::Minus,
                        Box::new(self.parse_expression_prefix()?)
                    ),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                })
            }
            _ => self.parse_expression_call()
        }
    }

    fn parse_expression_postfix(&mut self) -> Result<ASTExpr<'a>, ParserError> {
        let starting_token = self.current_token;

        let mut lhs = self.parse_expression_prefix()?;

        loop {
            lhs = match self.peek() {
                TokenKind::PlusPlus => {
                    self.advance();
                    ASTExpr {
                        kind: ASTExprKind::PostOp(Box::new(lhs), ASTOperator::Inc),
                        span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                    }
                },
                TokenKind::MinusMinus => {
                    self.advance();
                    ASTExpr {
                        kind: ASTExprKind::PostOp(Box::new(lhs), ASTOperator::Dec),
                        span: TokenSpan::new_rn_ex(starting_token, self.current_token)
                    }
                },
                _ => break
            }
        }

        Ok(lhs)
    }

    fn parse_expression_with_precedence(&mut self, min_precedence: OpPrecedence) -> Result<ASTExpr<'a>, ParserError> {
        let mut lhs = self.parse_expression_postfix()?;

        'loop_break: loop {
            let starting_token = self.current_token;
            let op_token = self.peek();
            let (op, op_precedence) = match op_token {
                TokenKind::Eq => (ASTOperator::Assign, OpPrecedence::Assignment),
                TokenKind::BinOpAssign(BinOpTokenKind::Plus) => (ASTOperator::PlusAssign, OpPrecedence::Assignment),
                TokenKind::BinOpAssign(BinOpTokenKind::Minus) => (ASTOperator::MinusAssign, OpPrecedence::Assignment),
                TokenKind::BinOpAssign(BinOpTokenKind::Star) => (ASTOperator::MulAssign, OpPrecedence::Assignment),
                TokenKind::BinOpAssign(BinOpTokenKind::Slash) => (ASTOperator::DivAssign, OpPrecedence::Assignment),
                TokenKind::OrOr => (ASTOperator::Or, OpPrecedence::Or),
                TokenKind::AndAnd => (ASTOperator::And, OpPrecedence::And),
                TokenKind::EqEq => (ASTOperator::Eq, OpPrecedence::Eq),
                TokenKind::Ls => (ASTOperator::Ls, OpPrecedence::Cmp),
                TokenKind::Gt => (ASTOperator::Gt, OpPrecedence::Cmp),
                TokenKind::LsEq => (ASTOperator::LsEq, OpPrecedence::Cmp),
                TokenKind::GtEq => (ASTOperator::GtEq, OpPrecedence::Cmp),
                TokenKind::BinOp(BinOpTokenKind::Plus) => (ASTOperator::Plus, OpPrecedence::Term),
                TokenKind::BinOp(BinOpTokenKind::Minus) => (ASTOperator::Minus, OpPrecedence::Term),
                TokenKind::BinOp(BinOpTokenKind::Star) => (ASTOperator::Mul, OpPrecedence::Factor),
                TokenKind::BinOp(BinOpTokenKind::Slash) => (ASTOperator::Div, OpPrecedence::Factor),
                _ => break 'loop_break
            };

            if op_precedence < min_precedence {
                break 'loop_break;
            }

            self.advance();

            let rhs = Box::new(self.parse_expression_with_precedence(op_precedence)?);
            lhs = ASTExpr {
                kind: ASTExprKind::BinOp(Box::new(lhs), op, rhs),
                span: TokenSpan::new_rn_ex(starting_token, self.current_token)
            };
        }

        Ok(lhs)
    }

    pub fn parse_expression(&mut self) -> Result<ASTExpr<'a>, ParserError> {
        self.parse_expression_with_precedence(OpPrecedence::Assignment)
    }

    pub fn parse_statement(&mut self) -> Result<ASTStatement<'a>, ParserError> {
        let starting_token = self.current_token;

        match self.peek() {
            TokenKind::Keyword(KeywordTokenKind::Let) => {
                self.advance();
                let name = self.tokens[self.current_token].string;
                self.advance_match(TokenKind::Ident)?;
                let type_info = if self.peek() == TokenKind::Colon {
                    self.advance();
                    Some(self.parse_type_info()?)
                }
                else {
                    None
                };
                let assignment = if self.peek() == TokenKind::Eq {
                    self.advance();
                    Some(Box::new(self.parse_expression()?))
                }
                else {
                    None
                };
                self.advance_match(TokenKind::Semicolon)?;

                Ok(ASTStatement {
                    kind: ASTStatementKind::Local(name, type_info, assignment),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                    ending: false
                })
            }
            _ => {
                let expr = self.parse_expression()?;
                let ending = self.peek() != TokenKind::Semicolon;
                if !ending {
                    self.advance();
                }

                Ok(ASTStatement {
                    kind: ASTStatementKind::Expression(Box::new(expr)),
                    span: TokenSpan::new_rn_ex(starting_token, self.current_token),
                    ending
                })
            }
        }
    }

    pub fn parse_statement_block(&mut self) -> Result<ASTStatementBlock<'a>, ParserError> {
        let starting_token = self.current_token;

        self.advance_match(TokenKind::OpeningDelim(DelimTokenKind::CBracket))?;
        let mut statements: Vec<ASTStatement> = vec![];
        while self.peek() != TokenKind::ClosingDelim(DelimTokenKind::CBracket) {
            if let Some(statement) = statements.last_mut() {
                if statement.ending {
                    self.statement_undo_ending(statement)?;
                }
            }
            statements.push(self.parse_statement()?);
        }
        self.advance();

        Ok(ASTStatementBlock {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            statements
        })
    }

    pub fn parse_path(&mut self) -> Result<ASTPath<'a>, ParserError> {
        let starting_token = self.current_token;

        let mut elements = vec![self.tokens[self.current_token].string];
        self.advance_match(TokenKind::Ident)?;
        while self.peek() == TokenKind::ColonColon {
            self.advance();
            elements.push(self.tokens[self.current_token].string);
            self.advance_match(TokenKind::Ident)?;
        }

        Ok(ASTPath {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            elements
        })
    }

    pub fn parse_type_info(&mut self) -> Result<ASTTypeInfo<'a>, ParserError> {
        let starting_token = self.current_token;

        let path = self.parse_path()?;
        let mut generics = vec![];
        let mut array_dim = 0;
        if self.peek() == TokenKind::Ls {
            self.advance();
            generics.push(self.parse_type_info()?);
            while self.peek() == TokenKind::Comma {
                self.advance();
                generics.push(self.parse_type_info()?);
            }
            self.advance_match(TokenKind::Gt)?;
        }
        while self.peek() == TokenKind::OpeningDelim(DelimTokenKind::SBracket) {
            self.advance();
            self.advance_match(TokenKind::ClosingDelim(DelimTokenKind::SBracket))?;
            array_dim += 1;
        }

        Ok(ASTTypeInfo {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            path,
            generics,
            array_dim
        })
    }

    pub fn parse_partial_type_info(&mut self) -> Result<ASTPartialTypeInfo<'a>, ParserError> {
        let starting_token = self.current_token;

        let path = self.parse_path()?;
        let mut generics = vec![];
        if self.peek() == TokenKind::Ls {
            self.advance();
            generics.push(self.parse_partial_type_info()?);
            while self.peek() == TokenKind::Comma {
                self.advance();
                generics.push(self.parse_partial_type_info()?);
            }
            self.advance_match(TokenKind::Gt)?;
        }

        Ok(ASTPartialTypeInfo {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            path,
            generics
        })
    }

    pub fn parse_mod(&mut self) -> Result<ASTMod<'a>, ParserError> {
        let starting_token = self.current_token;

        self.advance_match(TokenKind::Keyword(KeywordTokenKind::Mod))?;
        let path = self.parse_path()?;
        self.advance_match(TokenKind::Semicolon)?;
        Ok(ASTMod {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            path
        })
    }

    pub fn parse_use(&mut self) -> Result<ASTUse<'a>, ParserError> {
        let starting_token = self.current_token;

        self.advance_match(TokenKind::Keyword(KeywordTokenKind::Use))?;
        let path = self.parse_path()?;
        self.advance_match(TokenKind::Semicolon)?;
        Ok(ASTUse {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            path
        })
    }

    pub fn parse_visibility(&mut self) -> ASTVisibility {
        match self.peek() {
            TokenKind::Keyword(KeywordTokenKind::Pub) => {
                self.advance();
                ASTVisibility::Public
            }
            TokenKind::Keyword(KeywordTokenKind::Priv) => {
                self.advance();
                ASTVisibility::Private
            }
            _ => ASTVisibility::Module
        }
    }

    pub fn parse_modifiers(&mut self) -> Vec<ASTModifier> {
        let mut result = vec![];
        loop {
            result.push(match self.peek() {
                TokenKind::Keyword(KeywordTokenKind::Abstract) => ASTModifier::Abstract,
                TokenKind::Keyword(KeywordTokenKind::Static) => ASTModifier::Static,
                TokenKind::Keyword(KeywordTokenKind::Native) => ASTModifier::Native,
                _ => break
            });
            self.advance();
        };

        result
    }

    pub fn parse_generic_bound(&mut self) -> Result<ASTGenericBound<'a>, ParserError> {
        let starting_token = self.current_token;

        let name = self.tokens[self.current_token].string;
        self.advance_match(TokenKind::Ident)?;
        let mut super_requirements = vec![];
        let mut impl_requirements = vec![];
        if self.peek() == TokenKind::Colon {
            self.advance();
            super_requirements.push(self.parse_partial_type_info()?);
        }
        if self.peek() == TokenKind::Keyword(KeywordTokenKind::Impl) {
            self.advance();
            impl_requirements.push(self.parse_partial_type_info()?);
            while self.peek() == TokenKind::Comma {
                self.advance();
                impl_requirements.push(self.parse_partial_type_info()?);
            }
        }

        Ok(ASTGenericBound {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            name,
            super_requirements,
            impl_requirements
        })
    }

    pub fn parse_generic_bounds(&mut self) -> Result<Vec<ASTGenericBound<'a>>, ParserError> {
        if self.peek() == TokenKind::Ls {
            let mut bounds = vec![];
            self.advance();
            bounds.push(self.parse_generic_bound()?);
            while self.peek() == TokenKind::Comma {
                self.advance();
                bounds.push(self.parse_generic_bound()?);
            }
            self.advance_match(TokenKind::Gt)?;
            Ok(bounds)
        }
        else {
            Ok(vec![])
        }
    }

    pub fn parse_name_and_type(&mut self) -> Result<ASTNameAndType<'a>, ParserError> {
        let starting_token = self.current_token;

        let name = self.tokens[self.current_token].string;
        self.advance_match(TokenKind::Ident)?;
        self.advance_match(TokenKind::Colon)?;

        let type_info = self.parse_type_info()?;

        Ok(ASTNameAndType {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            name,
            type_info
        })
    }

    pub fn parse_member(&mut self) -> Result<ASTMember<'a>, ParserError> {
        let starting_token = self.current_token;

        let visibility = self.parse_visibility();
        let modifiers = self.parse_modifiers();

        let kind = if self.peek() == TokenKind::Keyword(KeywordTokenKind::Fn) {
            self.advance();
            let name = self.tokens[self.current_token].string;
            self.advance_match(TokenKind::Ident)?;
            self.advance_match(TokenKind::OpeningDelim(DelimTokenKind::Paren))?;
            let mut parameters = vec![];
            if self.peek() != TokenKind::ClosingDelim(DelimTokenKind::Paren) {
                parameters.push(self.parse_name_and_type()?);
                while self.peek() != TokenKind::ClosingDelim(DelimTokenKind::Paren) {
                    self.advance_match(TokenKind::Comma)?;
                    parameters.push(self.parse_name_and_type()?);
                }
            }
            self.advance();
            let name_and_type_start = self.current_token;
            let type_info = if self.peek() == TokenKind::Arrow {
                self.advance();
                self.parse_type_info()?
            }
            else {
                ASTTypeInfo {
                    span: TokenSpan::new(self.current_token, 0),
                    path: ASTPath {
                        span: TokenSpan::new(self.current_token, 0),
                        elements: vec!["void"]
                    },
                    generics: vec![],
                    array_dim: 0
                }
            };
            let name_and_type_end = self.current_token;
            let block = if self.peek() == TokenKind::Semicolon {
                self.advance();
                None
            }
            else {
                Some(self.parse_statement_block()?)
            };

            ASTMemberKind::Method {
                name_and_type: ASTNameAndType {
                    span: TokenSpan::new_rn_ex(name_and_type_start, name_and_type_end),
                    name,
                    type_info
                },
                parameters,
                block
            }
        }
        else {
            let name_and_type = self.parse_name_and_type()?;
            let expression = if self.peek() == TokenKind::Eq {
                self.advance();
                Some(self.parse_expression()?)
            }
            else {
                None
            };
            self.advance_match(TokenKind::Semicolon)?;

            ASTMemberKind::Field {
                name_and_type,
                expression
            }
        };

        Ok(ASTMember {
            kind,
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            visibility,
            modifiers
        })
    }

    pub fn parse_type(&mut self) -> Result<ASTType<'a>, ParserError> {
        let starting_token = self.current_token;

        let visibility = self.parse_visibility();
        let modifiers = self.parse_modifiers();
        let generics;
        let name;

        let kind = match self.peek() {
            TokenKind::Keyword(KeywordTokenKind::Class) => {
                self.advance();
                name = self.tokens[self.current_token].string;
                self.advance_match(TokenKind::Ident)?;
                generics = self.parse_generic_bounds()?;

                let super_class = if self.peek() == TokenKind::Colon {
                    self.advance();
                    Some(self.parse_partial_type_info()?)
                }
                else {
                    None
                };
                let mut impls = vec![];
                if self.peek() == TokenKind::Keyword(KeywordTokenKind::Impl) {
                    self.advance();
                    impls.push(self.parse_partial_type_info()?);
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        impls.push(self.parse_partial_type_info()?);
                    }
                }
                self.advance_match(TokenKind::OpeningDelim(DelimTokenKind::CBracket))?;
                let mut members = vec![];
                while self.peek() != TokenKind::ClosingDelim(DelimTokenKind::CBracket) {
                    members.push(self.parse_member()?);
                }
                self.advance();

                ASTTypeKind::Class {
                    super_class,
                    members,
                    impls
                }
            }
            _ => {
                self.advance();
                return Err(ParserError::new(TokenSpan::new_rn_ex(starting_token, self.current_token), self.peek()));
            }
        };

        Ok(ASTType {
            kind,
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            name,
            visibility,
            modifiers,
            generics
        })
    }

    pub fn parse(&mut self) -> Result<ASTRoot<'a>, ParserError> {
        let starting_token = self.current_token;

        let mod_decl = self.parse_mod()?;
        let mut use_decls = vec![];
        while self.peek() == TokenKind::Keyword(KeywordTokenKind::Use) {
            use_decls.push(self.parse_use()?);
        }
        let mut types = vec![];

        while self.peek() != TokenKind::EndOfFile {
            types.push(self.parse_type()?);
        }

        Ok(ASTRoot {
            span: TokenSpan::new_rn_ex(starting_token, self.current_token),
            types,
            mod_decl,
            use_decls
        })
    }
}

#[derive(Debug)]
pub struct ParserError {
    pub span: TokenSpan,
    pub got: TokenKind
}

impl ParserError {
    pub fn new(span: TokenSpan, got: TokenKind) -> Self {
        ParserError {
            span,
            got
        }
    }
}