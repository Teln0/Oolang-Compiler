#[derive(Eq, PartialEq)]
pub struct Ty {
    pub ty_def_id: usize,
    pub generics: Vec<Ty>,
    pub array_dim: usize
}

pub struct ClassTyDef<'a> {
    pub generics: Vec<usize>,
    pub name: &'a str,
    pub super_class: Option<Ty>,
    pub is_abstract: bool
}

pub enum PrimitiveTyDef {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,

    F32,
    F64,

    Bool,
    Char
}

pub struct GenericTyDef<'a> {
    pub name: &'a str,
    pub belongs_to: usize,
    pub super_requirements: Vec<Ty>,
    pub impl_requirements: Vec<Ty>
}

pub enum TyDef<'a> {
    Class(ClassTyDef<'a>),
    Primitive(PrimitiveTyDef),
    Generic(GenericTyDef<'a>)
}

impl<'a> TyDef<'a> {
    pub fn unwrap_class(self) -> ClassTyDef<'a> {
        match self {
            TyDef::Class(c) => c,
            _ => panic!()
        }
    }

    pub fn unwrap_class_ref(&self) -> &ClassTyDef<'a> {
        match self {
            TyDef::Class(c) => c,
            _ => panic!()
        }
    }

    pub fn unwrap_class_ref_mut(&mut self) -> &mut ClassTyDef<'a> {
        match self {
            TyDef::Class(c) => c,
            _ => panic!()
        }
    }

    pub fn unwrap_generic(self) -> GenericTyDef<'a> {
        match self {
            TyDef::Generic(c) => c,
            _ => panic!()
        }
    }

    pub fn unwrap_generic_ref(&self) -> &GenericTyDef<'a> {
        match self {
            TyDef::Generic(c) => c,
            _ => panic!()
        }
    }

    pub fn unwrap_generic_ref_mut(&mut self) -> &mut GenericTyDef<'a> {
        match self {
            TyDef::Generic(c) => c,
            _ => panic!()
        }
    }
}