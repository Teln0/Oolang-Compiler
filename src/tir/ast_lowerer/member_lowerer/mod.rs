use crate::tir::ast_lowerer::{ASTtoTIRLowerer, GenericContext, ASTtoTIRLowererError};
use crate::ast::{ASTMember, ASTModifier, ASTMemberKind, ASTExpr, ASTNameAndType, ASTExprKind, ASTOperator, ASTStatementBlock, ASTStatement, ASTStatementKind};
use crate::tir::{TIRMember, TIRExpr, TIRNameAndType, TIRMemberKind, TIRModifier, TIRExprKind, TIROperator, TIRStatementBlock, TIRStatement, TIRStatementKind};

impl<'a> ASTtoTIRLowerer<'a> {
    fn lower_ast_statement(&self, statement: &ASTStatement<'a>, generic_context: &GenericContext<'a, '_>) -> Result<TIRStatement<'a>, ASTtoTIRLowererError<'a>> {
        Ok(TIRStatement {
            kind: match &statement.kind {
                ASTStatementKind::Local(name, type_info, expr) =>
                    TIRStatementKind::Local(
                        name,
                        if let Some(type_info) = type_info {
                            Some(self.resolve_type_info(type_info, Some(generic_context))?)
                        } else {
                            None
                        },
                        if let Some(expr) = expr {
                            Some(Box::new(self.lower_ast_expr(expr, generic_context)?))
                        } else {
                            None
                        }
                    ),
                ASTStatementKind::Expression(expr) =>
                    TIRStatementKind::Expression(Box::new(self.lower_ast_expr(expr, generic_context)?))
            },
            span: statement.span.clone(),
            ending: statement.ending
        })
    }

    fn lower_ast_statement_block(&self, block: &ASTStatementBlock<'a>, generic_context: &GenericContext<'a, '_>) -> Result<TIRStatementBlock<'a>, ASTtoTIRLowererError<'a>> {
        Ok(TIRStatementBlock {
            statements: block.statements.iter().map(|s| self.lower_ast_statement(s, generic_context))
                .collect::<Result<Vec<TIRStatement<'a>>, ASTtoTIRLowererError<'a>>>()?,
            span: block.span.clone()
        })
    }

    fn lower_ast_operator(&self, operator: &ASTOperator) -> TIROperator {
        match operator {
            ASTOperator::Plus => TIROperator::Plus,
            ASTOperator::Minus => TIROperator::Minus,
            ASTOperator::Mul => TIROperator::Mul,
            ASTOperator::Div => TIROperator::Div,
            ASTOperator::PlusAssign => TIROperator::PlusAssign,
            ASTOperator::MinusAssign => TIROperator::MinusAssign,
            ASTOperator::MulAssign => TIROperator::MulAssign,
            ASTOperator::DivAssign => TIROperator::DivAssign,
            ASTOperator::Assign => TIROperator::Assign,
            ASTOperator::And => TIROperator::And,
            ASTOperator::Or => TIROperator::Or,
            ASTOperator::Eq => TIROperator::Eq,
            ASTOperator::NotEq => TIROperator::NotEq,
            ASTOperator::Gt => TIROperator::Gt,
            ASTOperator::GtEq => TIROperator::GtEq,
            ASTOperator::Ls => TIROperator::Ls,
            ASTOperator::LsEq => TIROperator::LsEq,
            ASTOperator::Inc => TIROperator::Inc,
            ASTOperator::Dec => TIROperator::Dec,
            ASTOperator::Not => TIROperator::Not
        }
    }

    fn lower_ast_expr(&self, expression: &ASTExpr<'a>, generic_context: &GenericContext<'a, '_>) -> Result<TIRExpr<'a>, ASTtoTIRLowererError<'a>> {
        Ok(TIRExpr {
            kind: match &expression.kind {
                ASTExprKind::Path(path) => TIRExprKind::TypeAccess(self.resolve_type_ref_index(&path.elements)?),
                ASTExprKind::Ident(ident) => TIRExprKind::VariableAccess(ident),
                ASTExprKind::StringLiteral(string) => TIRExprKind::StringLiteral(string),
                ASTExprKind::Num(number_literal) => TIRExprKind::Num(number_literal),
                ASTExprKind::Float(float_literal) => TIRExprKind::Float(float_literal),
                ASTExprKind::Boolean(boolean) => TIRExprKind::Boolean(*boolean),
                ASTExprKind::Null => TIRExprKind::Null,
                ASTExprKind::BinOp(left, op, right) =>
                    TIRExprKind::BinOp(
                        Box::new(self.lower_ast_expr(left, generic_context)?),
                        self.lower_ast_operator(op),
                        Box::new(self.lower_ast_expr(right, generic_context)?)
                    ),
                ASTExprKind::PreOp(op, expr) =>
                    TIRExprKind::PreOp(
                        self.lower_ast_operator(op),
                        Box::new(self.lower_ast_expr(expr, generic_context)?)
                    ),
                ASTExprKind::PostOp(expr, op) =>
                    TIRExprKind::PostOp(
                        Box::new(self.lower_ast_expr(expr, generic_context)?),
                        self.lower_ast_operator(op)
                    ),
                ASTExprKind::MemberAccess(expr, member) =>
                    TIRExprKind::MemberAccess(
                        Box::new(self.lower_ast_expr(expr, generic_context)?),
                        member
                    ),
                ASTExprKind::StaticAccess(expr, member) =>
                    TIRExprKind::StaticAccess(
                        Box::new(self.lower_ast_expr(expr, generic_context)?),
                        member
                    ),
                ASTExprKind::Call(expr, args) =>
                    TIRExprKind::Call(
                        Box::new(self.lower_ast_expr(expr, generic_context)?),
                        args.iter().map(|expr| { self.lower_ast_expr(expr, generic_context) })
                            .collect::<Result<Vec<TIRExpr<'a>>, ASTtoTIRLowererError<'a>>>()?
                    ),
                ASTExprKind::Indexing(expr, index) =>
                    TIRExprKind::Indexing(
                        Box::new(self.lower_ast_expr(expr, generic_context)?),
                        Box::new(self.lower_ast_expr(index, generic_context)?)
                    ),
                ASTExprKind::Block(statement_block) =>
                    TIRExprKind::Block(self.lower_ast_statement_block(statement_block, generic_context)?),
                ASTExprKind::IfElse(cond, then_block, else_block) =>
                    TIRExprKind::IfElse(
                        Box::new(self.lower_ast_expr(cond, generic_context)?),
                        self.lower_ast_statement_block(then_block, generic_context)?,
                        self.lower_ast_statement_block(else_block, generic_context)?
                    ),
                ASTExprKind::If(cond, then_block) =>
                    TIRExprKind::If(
                        Box::new(self.lower_ast_expr(cond, generic_context)?),
                        self.lower_ast_statement_block(then_block, generic_context)?
                    ),
                ASTExprKind::Loop(block) =>
                    TIRExprKind::Loop(self.lower_ast_statement_block(block, generic_context)?),
                ASTExprKind::While(cond, block) =>
                    TIRExprKind::While(
                        Box::new(self.lower_ast_expr(cond, generic_context)?),
                        self.lower_ast_statement_block(block, generic_context)?
                    ),
                ASTExprKind::Match() => {
                    todo!()
                }
                ASTExprKind::For() => {
                    todo!()
                }
            },
            span: expression.span.clone()
        })
    }

    fn lower_ast_name_and_type(&self, name_and_type: &ASTNameAndType<'a>, generic_context: &GenericContext<'a, '_>) -> Result<TIRNameAndType<'a>, ASTtoTIRLowererError<'a>> {
        Ok(TIRNameAndType {
            name: name_and_type.name,
            type_info: self.resolve_type_info(&name_and_type.type_info, Some(generic_context))?,
            span: name_and_type.span.clone()
        })
    }

    pub fn lower_ast_member(&self, member: &ASTMember<'a>, generic_context: &GenericContext<'a, '_>) -> Result<TIRMember<'a>, ASTtoTIRLowererError<'a>> {
        Ok(TIRMember {
            kind: match &member.kind {
                ASTMemberKind::Field { expression, name_and_type } => {
                    let expression = if let Some(expression) = expression {
                        Some(self.lower_ast_expr(expression, generic_context)?)
                    } else {
                        None
                    };
                    let name_and_type = self.lower_ast_name_and_type(name_and_type, generic_context)?;
                    TIRMemberKind::Field {
                        name_and_type,
                        expression
                    }
                },
                ASTMemberKind::Method { block, parameters, name_and_type } => {
                    let block = if let Some(block) = block {
                        Some(self.lower_ast_statement_block(block, generic_context)?)
                    } else {
                        None
                    };
                    let parameters = parameters.iter().map(|name_and_type| self.lower_ast_name_and_type(name_and_type, generic_context))
                        .collect::<Result<Vec<TIRNameAndType<'a>>, ASTtoTIRLowererError<'a>>>()?;
                    let name_and_type = self.lower_ast_name_and_type(name_and_type, generic_context)?;
                    TIRMemberKind::Method {
                        name_and_type,
                        block,
                        parameters
                    }
                }
            },
            span: member.span.clone(),
            modifiers: member.modifiers.iter().map(|m| { match m {
                ASTModifier::Static => TIRModifier::Static,
                ASTModifier::Abstract => TIRModifier::Abstract,
                ASTModifier::Native => TIRModifier::Native
            } }).collect()
        })
    }
}