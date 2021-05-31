use crate::ast::*;

pub trait ASTVisitor<'a> {
    fn walk_root(&mut self, obj: &ASTRoot<'a>) {
        self.walk_mod(&obj.mod_decl);

        for use_decl in &obj.use_decls {
            self.walk_use(use_decl);
        }

        for type_decl in &obj.types {
            self.walk_type(type_decl);
        }
    }

    fn walk_mod(&mut self, obj: &ASTMod<'a>) {
        self.walk_path(&obj.path);
    }

    fn walk_use(&mut self, obj: &ASTUse<'a>) {
        self.walk_path(&obj.path);
    }

    fn walk_modifier(&mut self, _obj: &ASTModifier) {}

    fn walk_visibility(&mut self, _obj: &ASTVisibility) {}

    fn walk_type(&mut self, obj: &ASTType<'a>) {
        match &obj.kind {
            ASTTypeKind::Class {
                members,
                impls,
                supers,
            } => {
                for member in members {
                    self.walk_member(member);
                }
                for impl_ in impls {
                    self.walk_partial_type_info(impl_);
                }
                for super_ in supers {
                    self.walk_partial_type_info(super_);
                }
            }
            ASTTypeKind::Inter {
                members,
                super_interfaces,
            } => {
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
        for generic_bound in &obj.generic_bounds {
            self.walk_generic_bound(generic_bound);
        }
        self.walk_visibility(&obj.visibility);
    }

    fn walk_generic_bound(&mut self, obj: &ASTGenericBound<'a>) {
        self.walk_path(&obj.path);
        for requirement in &obj.super_requirements {
            self.walk_partial_type_info(requirement);
        }
        for requirement in &obj.impl_requirements {
            self.walk_partial_type_info(requirement);
        }
    }

    fn walk_member(&mut self, obj: &ASTMember<'a>) {
        match &obj.kind {
            ASTMemberKind::Field {
                expression,
                name_and_type,
            } => {
                self.walk_name_and_type(name_and_type);
                if let Some(expression) = expression {
                    self.walk_expr(expression);
                }
            }
            ASTMemberKind::Method {
                name_and_type,
                block,
                parameters,
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

    fn walk_name_and_type(&mut self, obj: &ASTNameAndType<'a>) {
        self.walk_type_info(&obj.type_info);
    }

    fn walk_partial_type_info(&mut self, obj: &ASTPartialTypeInfo<'a>) {
        self.walk_path(&obj.path);
        for generic in &obj.generics {
            self.walk_partial_type_info(generic);
        }
    }

    fn walk_type_info(&mut self, obj: &ASTTypeInfo<'a>) {
        self.walk_path(&obj.path);
        for generic in &obj.generics {
            self.walk_type_info(generic);
        }
    }

    fn walk_path(&mut self, _obj: &ASTPath<'a>) {}

    fn walk_statement(&mut self, obj: &ASTStatement<'a>) {
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

    fn walk_statement_block(&mut self, obj: &ASTStatementBlock<'a>) {
        for statement in &obj.statements {
            self.walk_statement(statement);
        }
    }

    fn walk_expr(&mut self, obj: &ASTExpr<'a>) {
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
            ASTExprKind::IfElse(cond, block_if, block_else) => {
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

pub trait ASTVisitorMut<'a> {
    fn walk_root(&mut self, obj: &mut ASTRoot<'a>) {
        self.walk_mod(&mut obj.mod_decl);

        for use_decl in &mut obj.use_decls {
            self.walk_use(use_decl);
        }

        for type_decl in &mut obj.types {
            self.walk_type(type_decl);
        }
    }

    fn walk_mod(&mut self, obj: &mut ASTMod<'a>) {
        self.walk_path(&mut obj.path);
    }

    fn walk_use(&mut self, obj: &mut ASTUse<'a>) {
        self.walk_path(&mut obj.path);
    }

    fn walk_modifier(&mut self, _obj: &mut ASTModifier) {}

    fn walk_visibility(&mut self, _obj: &mut ASTVisibility) {}

    fn walk_type(&mut self, obj: &mut ASTType<'a>) {
        walk_type_default_mut(self, obj);
    }

    fn walk_generic_bound(&mut self, obj: &mut ASTGenericBound<'a>) {
        self.walk_path(&mut obj.path);
        for requirement in &mut obj.super_requirements {
            self.walk_partial_type_info(requirement);
        }
        for requirement in &mut obj.impl_requirements {
            self.walk_partial_type_info(requirement);
        }
    }

    fn walk_member(&mut self, obj: &mut ASTMember<'a>) {
        match &mut obj.kind {
            ASTMemberKind::Field {
                expression,
                name_and_type,
            } => {
                self.walk_name_and_type(name_and_type);
                if let Some(expression) = expression {
                    self.walk_expr(expression);
                }
            }
            ASTMemberKind::Method {
                name_and_type,
                block,
                parameters,
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

        for modifier in &mut obj.modifiers {
            self.walk_modifier(modifier);
        }
        self.walk_visibility(&mut obj.visibility);
    }

    fn walk_name_and_type(&mut self, obj: &mut ASTNameAndType<'a>) {
        self.walk_type_info(&mut obj.type_info);
    }

    fn walk_partial_type_info(&mut self, obj: &mut ASTPartialTypeInfo<'a>) {
        self.walk_path(&mut obj.path);
        for generic in &mut obj.generics {
            self.walk_partial_type_info(generic);
        }
    }

    fn walk_type_info(&mut self, obj: &mut ASTTypeInfo<'a>) {
        self.walk_path(&mut obj.path);
        for generic in &mut obj.generics {
            self.walk_type_info(generic);
        }
    }

    fn walk_path(&mut self, _obj: &mut ASTPath<'a>) {}

    fn walk_statement(&mut self, obj: &mut ASTStatement<'a>) {
        match &mut obj.kind {
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

    fn walk_statement_block(&mut self, obj: &mut ASTStatementBlock<'a>) {
        for statement in &mut obj.statements {
            self.walk_statement(statement);
        }
    }

    fn walk_expr(&mut self, obj: &mut ASTExpr<'a>) {
        match &mut obj.kind {
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
            ASTExprKind::IfElse(cond, block_if, block_else) => {
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

pub fn walk_type_default_mut<'a, T: ASTVisitorMut<'a> + ?Sized>(visitor: &mut T, obj: &mut ASTType<'a>) {
    match &mut obj.kind {
        ASTTypeKind::Class {
            members,
            impls,
            supers,
        } => {
            for member in members {
                visitor.walk_member(member);
            }
            for impl_ in impls {
                visitor.walk_partial_type_info(impl_);
            }
            for super_ in supers {
                visitor.walk_partial_type_info(super_);
            }
        }
        ASTTypeKind::Inter {
            members,
            super_interfaces,
        } => {
            for member in members {
                visitor.walk_member(member);
            }
            for inter in super_interfaces {
                visitor.walk_partial_type_info(inter);
            }
        }
        ASTTypeKind::Enum { .. } => {
            // TODO
        }
        ASTTypeKind::Impl { members } => {
            for member in members {
                visitor.walk_member(member);
            }
        }
    }

    for modifier in &mut obj.modifiers {
        visitor.walk_modifier(modifier);
    }
    for generic_bound in &mut obj.generic_bounds {
        visitor.walk_generic_bound(generic_bound);
    }
    visitor.walk_visibility(&mut obj.visibility);
}