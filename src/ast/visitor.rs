use crate::ast::*;

pub trait ASTVisitor {
    fn walk_root(&mut self, obj: &ASTRoot) {
        self.walk_mod(&obj.mod_decl);

        for use_decl in &obj.use_decls {
            self.walk_use(use_decl);
        }
    }

    fn walk_mod(&mut self, obj: &ASTMod) {
        self.walk_path(&obj.path);
    }

    fn walk_use(&mut self, obj: &ASTUse) {
        self.walk_path(&obj.path);
    }

    fn walk_modifier(&mut self, _obj: &ASTModifier) {

    }

    fn walk_visibility(&mut self, _obj: &ASTVisibility) {

    }

    fn walk_type(&mut self, obj: &ASTType) {
        match &obj.kind {
            ASTTypeKind::Class { members, impls, super_class} => {
                for member in members {
                    self.walk_member(member);
                }
                for impl_ in impls {
                    self.walk_partial_type_info(impl_);
                }
                if let Some(super_class) = super_class {
                    self.walk_partial_type_info(super_class);
                }
            }
            ASTTypeKind::Inter { members, super_interfaces } => {
                for member in members {
                    self.walk_member(member);
                }
                for inter in super_interfaces {
                    self.walk_partial_type_info(inter);
                }
            }
            ASTTypeKind::Enum { .. } => {
                // TODO
            }
            ASTTypeKind::Impl { members } => {
                for member in members {
                    self.walk_member(member);
                }
            }
        }

        for modifier in &obj.modifiers {
            self.walk_modifier(modifier);
        }
        for generic in &obj.generics {
            self.walk_generic_bound(generic);
        }
        self.walk_visibility(&obj.visibility);
    }

    fn walk_generic_bound(&mut self, obj: &ASTGenericBound) {
        for requirement in &obj.super_requirements {
            self.walk_partial_type_info(requirement);
        }
        for requirement in &obj.impl_requirements {
            self.walk_partial_type_info(requirement);
        }
    }

    fn walk_member(&mut self, obj: &ASTMember) {
        match &obj.kind {
            ASTMemberKind::Field { expression, name_and_type } => {
                self.walk_name_and_type(name_and_type);
                if let Some(expression) = expression {
                    self.walk_expr(expression);
                }
            }
            ASTMemberKind::Method {
                name_and_type,
                block,
                parameters
            } => {
                self.walk_name_and_type(name_and_type);
                if let Some(block) = block {
                    self.walk_statement_block(block);
                }
                for parameter in parameters {
                    self.walk_name_and_type(parameter);
                }
            }
        }

        for modifier in &obj.modifiers {
            self.walk_modifier(modifier);
        }
        self.walk_visibility(&obj.visibility);
    }

    fn walk_name_and_type(&mut self, obj: &ASTNameAndType) {
        self.walk_type_info(&obj.type_info);
    }

    fn walk_partial_type_info(&mut self, obj: &ASTPartialTypeInfo) {
        self.walk_path(&obj.path);
        for generic in &obj.generics {
            self.walk_partial_type_info(generic);
        }
    }

    fn walk_type_info(&mut self, obj: &ASTTypeInfo) {
        self.walk_path(&obj.path);
        for generic in &obj.generics {
            self.walk_type_info(generic);
        }
    }

    fn walk_path(&mut self, _obj: &ASTPath) {

    }

    fn walk_statement(&mut self, obj: &ASTStatement) {
        match &obj.kind {
            ASTStatementKind::Local(_, type_info, local) => {
                if let Some(type_info) = type_info {
                    self.walk_type_info(type_info);
                }
                if let Some(local) = local {
                    self.walk_expr(local);
                }
            }
            ASTStatementKind::Expression(expr) => {
                self.walk_expr(expr);
            }
        }
    }

    fn walk_statement_block(&mut self, obj: &ASTStatementBlock) {
        for statement in &obj.statements {
            self.walk_statement(statement);
        }
    }

    fn walk_expr(&mut self, obj: &ASTExpr) {
        match &obj.kind {
            ASTExprKind::Ident(_) => {}
            ASTExprKind::StringLiteral(_) => {}
            ASTExprKind::Num(_) => {}
            ASTExprKind::Float(_) => {}
            ASTExprKind::Boolean(_) => {}
            ASTExprKind::Null => {}
            ASTExprKind::BinOp(left, _, right) => {
                self.walk_expr(left);
                self.walk_expr(right);
            }
            ASTExprKind::PreOp(_, expr) => {
                self.walk_expr(expr);
            }
            ASTExprKind::PostOp(expr, _) => {
                self.walk_expr(expr);
            }
            ASTExprKind::MemberAccess(expr, _) => {
                self.walk_expr(expr);
            }
            ASTExprKind::StaticAccess(expr, _) => {
                self.walk_expr(expr);
            }
            ASTExprKind::Call(expr, args) => {
                self.walk_expr(expr);
                for arg in args {
                    self.walk_expr(arg);
                }
            }
            ASTExprKind::Indexing(expr, index) => {
                self.walk_expr(expr);
                self.walk_expr(index);
            }
            ASTExprKind::Block(block) => {
                self.walk_statement_block(block);
            }
            ASTExprKind::IfElse(
                cond,
                block_if,
                block_else
            ) => {
                self.walk_expr(cond);
                self.walk_statement_block(block_if);
                self.walk_statement_block(block_else);
            }
            ASTExprKind::Loop(block) => {
                self.walk_statement_block(block);
            }
            ASTExprKind::Match() => { /* TODO*/ }
            ASTExprKind::If(_, block) => {
                self.walk_statement_block(block);
            }
            ASTExprKind::While(_, block) => {
                self.walk_statement_block(block);
            }
            ASTExprKind::For() => {}
        }
    }
}