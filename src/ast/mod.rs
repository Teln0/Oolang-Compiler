pub mod visitor;

use crate::reporting::TokenSpan;

pub struct ASTRoot<'a> {
    pub span: TokenSpan,
    pub mod_decl: ASTMod<'a>,
    pub use_decls: Vec<ASTUse<'a>>,
    pub types: Vec<ASTType<'a>>
}

pub struct ASTMod<'a> {
    pub span: TokenSpan,
    pub path: ASTPath<'a>
}

pub struct ASTUse<'a> {
    pub span: TokenSpan,
    pub path: ASTPath<'a>
}

pub enum ASTModifier {
    Static,
    Abstract,
    Native
}

pub enum ASTVisibility {
    Public,
    Module,
    Private
}

pub enum ASTTypeKind<'a> {
    Class {
        members: Vec<ASTMember<'a>>,
        super_class: Option<ASTPartialTypeInfo<'a>>,
        impls: Vec<ASTPartialTypeInfo<'a>>
    },
    Inter {
        members: Vec<ASTMember<'a>>, // should only be functions
        super_interfaces: Vec<ASTPartialTypeInfo<'a>>
    },
    Enum {
        // TODO
    },
    Impl {
        members: Vec<ASTMember<'a>> // should only be functions
    }
}

pub struct ASTType<'a> {
    pub kind: ASTTypeKind<'a>,
    pub span: TokenSpan,
    pub visibility: ASTVisibility,
    pub modifiers: Vec<ASTModifier>,
    pub name: &'a str,
    pub generics: Vec<ASTGenericBound<'a>>
}

pub struct ASTGenericBound<'a> {
    pub span: TokenSpan,
    pub name: &'a str,
    pub super_requirements: Vec<ASTPartialTypeInfo<'a>>,
    pub impl_requirements: Vec<ASTPartialTypeInfo<'a>>,
}

pub enum ASTMemberKind<'a> {
    Field {
        name_and_type: ASTNameAndType<'a>,
        expression: Option<ASTExpr<'a>>
    },
    Method {
        name_and_type: ASTNameAndType<'a>,
        parameters: Vec<ASTNameAndType<'a>>,
        block: Option<ASTStatementBlock<'a>>
    }
}

pub struct ASTMember<'a> {
    pub kind: ASTMemberKind<'a>,
    pub span: TokenSpan,
    pub visibility: ASTVisibility,
    pub modifiers: Vec<ASTModifier>
}

pub struct ASTNameAndType<'a> {
    pub span: TokenSpan,
    pub type_info: ASTTypeInfo<'a>,
    pub name: &'a str
}

#[derive(Clone)]
pub struct ASTPartialTypeInfo<'a> {
    pub span: TokenSpan,
    pub path: ASTPath<'a>,
    pub generics: Vec<ASTPartialTypeInfo<'a>>
}

#[derive(Clone)]
pub struct ASTTypeInfo<'a> {
    pub span: TokenSpan,
    pub path: ASTPath<'a>,
    pub generics: Vec<ASTTypeInfo<'a>>,
    pub array_dim: usize
}

#[derive(Clone)]
pub struct ASTPath<'a> {
    pub span: TokenSpan,
    pub elements: Vec<&'a str>
}

#[derive(Debug)]
pub enum ASTOperator {
    Plus,
    Minus,
    Mul,
    Div,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    Assign,
    And,
    Or,
    Eq,
    NotEq,
    Gt,
    GtEq,
    Ls,
    LsEq,

    Inc,
    Dec,
    Not
}

pub enum ASTStatementKind<'a> {
    Local(&'a str, Option<ASTTypeInfo<'a>>, Option<Box<ASTExpr<'a>>>),
    Expression(Box<ASTExpr<'a>>)
}

pub struct ASTStatement<'a> {
    pub kind: ASTStatementKind<'a>,
    pub span: TokenSpan,
    pub ending: bool
}

pub struct ASTStatementBlock<'a> {
    pub span: TokenSpan,
    pub statements: Vec<ASTStatement<'a>>
}

pub enum ASTExprKind<'a> {
    Ident(&'a str),
    StringLiteral(&'a str),
    Num(&'a str),
    Float(&'a str),
    Boolean(bool),
    Null,

    BinOp(Box<ASTExpr<'a>>, ASTOperator, Box<ASTExpr<'a>>),
    PreOp(ASTOperator, Box<ASTExpr<'a>>),
    PostOp(Box<ASTExpr<'a>>, ASTOperator),

    MemberAccess(Box<ASTExpr<'a>>, &'a str),
    StaticAccess(Box<ASTExpr<'a>>, &'a str),
    Call(Box<ASTExpr<'a>>, Vec<ASTExpr<'a>>),
    Indexing(Box<ASTExpr<'a>>, Box<ASTExpr<'a>>),

    Block(ASTStatementBlock<'a>),
    IfElse(Box<ASTExpr<'a>>, ASTStatementBlock<'a>, ASTStatementBlock<'a>),
    If(Box<ASTExpr<'a>>, ASTStatementBlock<'a>),
    Loop(ASTStatementBlock<'a>),
    While(Box<ASTExpr<'a>>, ASTStatementBlock<'a>),
    Match(/* TODO */),
    For(/* TODO */)
}

pub struct ASTExpr<'a> {
    pub kind: ASTExprKind<'a>,
    pub span: TokenSpan
}

impl<'a> ASTPartialTypeInfo<'a> {
    pub fn into_type_info(&self) -> ASTTypeInfo<'a> {
        ASTTypeInfo {
            span: self.span,
            path: self.path.clone(),
            generics: self.generics.iter().map(|g| g.into_type_info()).collect(),
            array_dim: 0
        }
    }
}