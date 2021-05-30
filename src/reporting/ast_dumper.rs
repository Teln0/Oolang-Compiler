use crate::ast::visitor::ASTVisitor;
use crate::ast::{
    ASTExpr, ASTExprKind, ASTGenericBound, ASTMember, ASTMemberKind, ASTModifier,
    ASTPartialTypeInfo, ASTPath, ASTRoot, ASTStatement, ASTStatementBlock, ASTStatementKind,
    ASTType, ASTTypeInfo, ASTTypeKind, ASTVisibility,
};
use crate::reporting::string_tree::StringTree;

struct ASTDumperVisitor {
    tree: StringTree,
}

impl ASTDumperVisitor {
    pub fn new(branch_name: String) -> Self {
        ASTDumperVisitor {
            tree: StringTree::new(branch_name),
        }
    }
}

impl ASTVisitor for ASTDumperVisitor {
    fn walk_expr(&mut self, obj: &ASTExpr) {
        match &obj.kind {
            ASTExprKind::Ident(s) => {
                self.tree.add_branch(s);
            }
            ASTExprKind::StringLiteral(s) => {
                self.tree.add_branch(s);
            }
            ASTExprKind::Num(s) => {
                self.tree.add_branch(s);
            }
            ASTExprKind::Float(s) => {
                self.tree.add_branch(s);
            }
            ASTExprKind::Boolean(b) => {
                self.tree.add_branch(if *b { "true" } else { "false" });
            }
            ASTExprKind::Null => {
                self.tree.add_branch("null");
            }
            ASTExprKind::BinOp(left, op, right) => {
                let mut branch = ASTDumperVisitor::new(format!("{:?}", op));
                branch.walk_expr(left);
                branch.walk_expr(right);
                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::PreOp(op, expr) => {
                let mut branch = ASTDumperVisitor::new(format!("(prefix) {:?}", op));
                branch.walk_expr(expr);
                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::PostOp(expr, op) => {
                let mut branch = ASTDumperVisitor::new(format!("(postfix) {:?}", op));
                branch.walk_expr(expr);
                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::MemberAccess(expr, member) => {
                let mut branch = ASTDumperVisitor::new(format!("(member access) {:?}", member));
                branch.walk_expr(expr);
                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::StaticAccess(expr, static_member) => {
                let mut branch =
                    ASTDumperVisitor::new(format!("(static access) {:?}", static_member));
                branch.walk_expr(expr);
                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::Call(expr, args) => {
                let mut branch = ASTDumperVisitor::new(format!("call"));

                let mut branch_inner = ASTDumperVisitor::new(format!("on"));
                branch_inner.walk_expr(expr);
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("args"));
                for arg in args {
                    branch_inner.walk_expr(arg);
                }
                branch.tree.add_tree_branch(branch_inner.tree);

                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::Indexing(expr, index) => {
                let mut branch = ASTDumperVisitor::new(format!("indexing"));

                let mut branch_inner = ASTDumperVisitor::new(format!("on"));
                branch_inner.walk_expr(expr);
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("index"));
                branch_inner.walk_expr(index);
                branch.tree.add_tree_branch(branch_inner.tree);

                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::Block(block) => {
                let mut branch = ASTDumperVisitor::new(format!("block"));
                branch.walk_statement_block(block);
                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::IfElse(cond, block_if, block_else) => {
                let mut branch = ASTDumperVisitor::new(format!("if else"));

                let mut branch_inner = ASTDumperVisitor::new(format!("condition"));
                branch_inner.walk_expr(cond);
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("if"));
                branch_inner.walk_statement_block(block_if);
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("else"));
                branch_inner.walk_statement_block(block_else);
                branch.tree.add_tree_branch(branch_inner.tree);

                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::Loop(block) => {
                let mut branch = ASTDumperVisitor::new(format!("loop"));
                branch.walk_statement_block(block);
                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::Match() => {}
            ASTExprKind::If(cond, block) => {
                let mut branch = ASTDumperVisitor::new(format!("if"));

                let mut branch_inner = ASTDumperVisitor::new(format!("condition"));
                branch_inner.walk_expr(cond);
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("then"));
                branch_inner.walk_statement_block(block);
                branch.tree.add_tree_branch(branch_inner.tree);

                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::While(cond, block) => {
                let mut branch = ASTDumperVisitor::new(format!("while"));

                let mut branch_inner = ASTDumperVisitor::new(format!("condition"));
                branch_inner.walk_expr(cond);
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("do"));
                branch_inner.walk_statement_block(block);
                branch.tree.add_tree_branch(branch_inner.tree);

                self.tree.add_tree_branch(branch.tree);
            }
            ASTExprKind::For() => {}
        }
    }

    fn walk_statement(&mut self, obj: &ASTStatement) {
        match &obj.kind {
            ASTStatementKind::Local(name, type_info, expr) => {
                let mut branch = ASTDumperVisitor::new(format!("let {}", name));
                if let Some(type_info) = type_info {
                    let mut branch_inner = ASTDumperVisitor::new(format!("ty"));
                    branch_inner.walk_type_info(type_info);
                    branch.tree.add_tree_branch(branch_inner.tree);
                }
                if let Some(expr) = expr {
                    let mut branch_inner = ASTDumperVisitor::new(format!("assignment"));
                    branch_inner.walk_expr(expr);
                    branch.tree.add_tree_branch(branch_inner.tree);
                }
                self.tree.add_tree_branch(branch.tree);
            }
            ASTStatementKind::Expression(expr) => {
                let mut branch = ASTDumperVisitor::new(format!(
                    "expression statement {}",
                    if obj.ending { "(ending)" } else { "" }
                ));
                branch.walk_expr(expr);
                self.tree.add_tree_branch(branch.tree);
            }
        }
    }

    fn walk_statement_block(&mut self, obj: &ASTStatementBlock) {
        for statement in &obj.statements {
            self.walk_statement(statement);
        }
    }

    fn walk_type_info(&mut self, obj: &ASTTypeInfo) {
        let mut branch = ASTDumperVisitor::new(format!("ty info"));

        let mut branch_inner = ASTDumperVisitor::new(format!("path"));
        branch_inner.walk_path(&obj.path);
        branch.tree.add_tree_branch(branch_inner.tree);

        let mut branch_inner = ASTDumperVisitor::new(format!("generics"));
        for generic in &obj.generics {
            branch_inner.walk_type_info(generic);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        branch
            .tree
            .add_branch(&format!("array dim {}", obj.array_dim));

        self.tree.add_tree_branch(branch.tree);
    }

    fn walk_partial_type_info(&mut self, obj: &ASTPartialTypeInfo) {
        let mut branch = ASTDumperVisitor::new(format!("partial ty info"));

        let mut branch_inner = ASTDumperVisitor::new(format!("path"));
        branch_inner.walk_path(&obj.path);
        branch.tree.add_tree_branch(branch_inner.tree);

        let mut branch_inner = ASTDumperVisitor::new(format!("generics"));
        for generic in &obj.generics {
            branch_inner.walk_partial_type_info(generic);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        self.tree.add_tree_branch(branch.tree);
    }

    fn walk_path(&mut self, obj: &ASTPath) {
        self.tree.add_branch(&obj.elements.join("::"));
    }

    fn walk_root(&mut self, obj: &ASTRoot) {
        let mut branch = ASTDumperVisitor::new(format!("ast root"));

        let mut branch_inner = ASTDumperVisitor::new(format!("mod"));
        branch_inner.walk_mod(&obj.mod_decl);
        branch.tree.add_tree_branch(branch_inner.tree);

        let mut branch_inner = ASTDumperVisitor::new(format!("uses"));
        for use_decl in &obj.use_decls {
            branch_inner.walk_use(use_decl);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        let mut branch_inner = ASTDumperVisitor::new(format!("types"));
        for type_decl in &obj.types {
            branch_inner.walk_type(type_decl);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        self.tree.add_tree_branch(branch.tree);
    }

    fn walk_visibility(&mut self, obj: &ASTVisibility) {
        match obj {
            ASTVisibility::Private => self.tree.add_branch("private"),
            ASTVisibility::Module => self.tree.add_branch("module"),
            ASTVisibility::Public => self.tree.add_branch("public"),
        }
    }

    fn walk_modifier(&mut self, obj: &ASTModifier) {
        match obj {
            ASTModifier::Static => self.tree.add_branch("static"),
            ASTModifier::Abstract => self.tree.add_branch("abstract"),
            ASTModifier::Native => self.tree.add_branch("native"),
        }
    }

    fn walk_generic_bound(&mut self, obj: &ASTGenericBound) {
        let mut branch = ASTDumperVisitor::new(format!("generic {}", obj.name));

        let mut branch_inner = ASTDumperVisitor::new(format!("super_requirements"));
        for requirement in &obj.super_requirements {
            branch_inner.walk_partial_type_info(requirement);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        let mut branch_inner = ASTDumperVisitor::new(format!("impl_requirements"));
        for requirement in &obj.impl_requirements {
            branch_inner.walk_partial_type_info(requirement);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        self.tree.add_tree_branch(branch.tree);
    }

    fn walk_member(&mut self, obj: &ASTMember) {
        let branch = match &obj.kind {
            ASTMemberKind::Field {
                name_and_type,
                expression,
            } => {
                let mut branch = ASTDumperVisitor::new(format!("field {}", name_and_type.name));

                let mut branch_inner = ASTDumperVisitor::new(format!("ty"));
                branch_inner.walk_type_info(&name_and_type.type_info);
                branch.tree.add_tree_branch(branch_inner.tree);

                if let Some(expression) = expression {
                    let mut branch_inner = ASTDumperVisitor::new(format!("assigned value"));
                    branch_inner.walk_expr(expression);
                    branch.tree.add_tree_branch(branch_inner.tree);
                }

                branch
            }
            ASTMemberKind::Method {
                name_and_type,
                parameters,
                block,
            } => {
                let mut branch = ASTDumperVisitor::new(format!("method {}", name_and_type.name));

                let mut branch_inner = ASTDumperVisitor::new(format!("return ty"));
                branch_inner.walk_type_info(&name_and_type.type_info);
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("parameters"));
                for parameter in parameters {
                    let mut branch_inner_inner =
                        ASTDumperVisitor::new(format!("parameter {}", parameter.name));
                    branch_inner_inner.walk_type_info(&parameter.type_info);
                    branch_inner.tree.add_tree_branch(branch_inner_inner.tree);
                }
                branch.tree.add_tree_branch(branch_inner.tree);

                if let Some(block) = block {
                    let mut branch_inner = ASTDumperVisitor::new(format!("block"));
                    branch_inner.walk_statement_block(block);
                    branch.tree.add_tree_branch(branch_inner.tree);
                }

                branch
            }
        };

        self.tree.add_tree_branch(branch.tree);
    }

    fn walk_type(&mut self, obj: &ASTType) {
        let mut branch = ASTDumperVisitor::new(format!(
            "{} {}",
            obj.name,
            match obj.kind {
                ASTTypeKind::Class { .. } => "class",
                ASTTypeKind::Inter { .. } => "inter",
                ASTTypeKind::Enum { .. } => "enum",
                ASTTypeKind::Impl { .. } => "impl",
            }
        ));

        let mut branch_inner = ASTDumperVisitor::new(format!("visibility"));
        branch_inner.walk_visibility(&obj.visibility);
        branch.tree.add_tree_branch(branch_inner.tree);

        let mut branch_inner = ASTDumperVisitor::new(format!("modifiers"));
        for modifier in &obj.modifiers {
            branch_inner.walk_modifier(modifier);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        let mut branch_inner = ASTDumperVisitor::new(format!("generics"));
        for generic in &obj.generics {
            branch_inner.tree.add_branch(generic);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        let mut branch_inner = ASTDumperVisitor::new(format!("generics_bounds"));
        for generic_bound in &obj.generic_bounds {
            branch_inner.walk_generic_bound(generic_bound);
        }
        branch.tree.add_tree_branch(branch_inner.tree);

        match &obj.kind {
            ASTTypeKind::Class {
                supers,
                impls,
                members,
            } => {
                let mut branch_inner = ASTDumperVisitor::new(format!("supers"));
                for super_decl in supers {
                    branch_inner.walk_partial_type_info(super_decl);
                }
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("impls"));
                for impl_decl in impls {
                    branch_inner.walk_partial_type_info(impl_decl);
                }
                branch.tree.add_tree_branch(branch_inner.tree);

                let mut branch_inner = ASTDumperVisitor::new(format!("members"));
                for member in members {
                    branch_inner.walk_member(member);
                }
                branch.tree.add_tree_branch(branch_inner.tree);
            }
            ASTTypeKind::Inter { .. } => {}
            ASTTypeKind::Enum { .. } => {}
            ASTTypeKind::Impl { .. } => {}
        }

        self.tree.add_tree_branch(branch.tree);
    }
}

pub fn dump_from_root(root: &ASTRoot) {
    let mut visitor = ASTDumperVisitor::new("root".to_string());
    visitor.walk_root(root);
    visitor.tree.dump();
}

pub fn dump_from_expr(expr: &ASTExpr) {
    let mut visitor = ASTDumperVisitor::new("expr".to_string());
    visitor.walk_expr(expr);
    visitor.tree.dump();
}
