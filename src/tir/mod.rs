use crate::reporting::TokenSpan;
use std::hash::{Hash, Hasher};

pub mod ast_lowerer;

pub struct TIRRoot<'a> {
    pub span: TokenSpan,
    pub types: Vec<TIRType<'a>>,
}

#[derive(Debug)]
pub enum TIRModifier {
    Static,
    Abstract,
    Native,
}

pub enum TIRTypeKind<'a> {
    Class {
        members: Vec<TIRMember<'a>>,
        super_class: Option<TIRTypeInfo>,
    },
}

pub struct TIRType<'a> {
    pub kind: TIRTypeKind<'a>,
    pub span: TokenSpan,
    pub type_ref_index: usize,
}

pub enum TIRMemberKind<'a> {
    Field {
        name_and_type: TIRNameAndType<'a>,
        expression: Option<TIRExpr<'a>>,
    },
    Method {
        name_and_type: TIRNameAndType<'a>,
        parameters: Vec<TIRNameAndType<'a>>,
        block: Option<TIRStatementBlock<'a>>,
    },
}

pub struct TIRMember<'a> {
    pub kind: TIRMemberKind<'a>,
    pub span: TokenSpan,
    pub modifiers: Vec<TIRModifier>,
}

pub struct TIRNameAndType<'a> {
    pub span: TokenSpan,
    pub type_info: TIRTypeInfo,
    pub name: &'a str,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PrimitiveType {
    Void,
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
    F64,
    F32,
    Boolean,
    Character,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TIRTypeInfoKind {
    TypeRef {
        type_ref_index: usize,
        generics: Vec<TIRTypeInfo>,
        array_dim: usize
    },
    Generic {
        type_ref_index: usize,
        generic_index: usize,
        array_dim: usize
    },
    Primitive {
        primitive: PrimitiveType,
        array_dim: usize
    }
}

#[derive(Clone, Debug)]
pub struct TIRTypeInfo {
    pub kind: TIRTypeInfoKind,
    pub span: TokenSpan,
}

impl PartialEq<Self> for TIRTypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for TIRTypeInfo {}

impl Hash for TIRTypeInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

#[derive(Clone)]
pub struct TIRPath<'a> {
    pub span: TokenSpan,
    pub elements: Vec<&'a str>,
}

#[derive(Debug, Clone)]
pub enum TIROperator {
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
    Not,
}

#[derive(Clone)]
pub enum TIRStatementKind<'a> {
    Local(&'a str, Option<TIRTypeInfo>, Option<Box<TIRExpr<'a>>>),
    Expression(Box<TIRExpr<'a>>),
}

#[derive(Clone)]
pub struct TIRStatement<'a> {
    pub kind: TIRStatementKind<'a>,
    pub span: TokenSpan,
    pub ending: bool,
}

#[derive(Clone)]
pub struct TIRStatementBlock<'a> {
    pub span: TokenSpan,
    pub statements: Vec<TIRStatement<'a>>,
}

#[derive(Clone)]
pub enum TIRExprKind<'a> {
    StringLiteral(&'a str),
    Num(&'a str),
    Float(&'a str),
    Boolean(bool),
    Null,

    BinOp(Box<TIRExpr<'a>>, TIROperator, Box<TIRExpr<'a>>),
    PreOp(TIROperator, Box<TIRExpr<'a>>),
    PostOp(Box<TIRExpr<'a>>, TIROperator),

    TypeAccess(usize),
    VariableAccess(&'a str),

    MemberAccess(Box<TIRExpr<'a>>, &'a str),
    StaticAccess(Box<TIRExpr<'a>>, &'a str),
    Call(Box<TIRExpr<'a>>, Vec<TIRExpr<'a>>),
    Indexing(Box<TIRExpr<'a>>, Box<TIRExpr<'a>>),

    Block(TIRStatementBlock<'a>),
    IfElse(
        Box<TIRExpr<'a>>,
        TIRStatementBlock<'a>,
        TIRStatementBlock<'a>,
    ),
    If(Box<TIRExpr<'a>>, TIRStatementBlock<'a>),
    Loop(TIRStatementBlock<'a>),
    While(Box<TIRExpr<'a>>, TIRStatementBlock<'a>),
    Match(/* TODO */),
    For(/* TODO */),
}

#[derive(Clone)]
pub struct TIRExpr<'a> {
    pub kind: TIRExprKind<'a>,
    pub span: TokenSpan,
}