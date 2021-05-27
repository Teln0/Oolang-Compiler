#[derive(Clone, Eq, PartialEq)]
pub struct RealTypeInfo {
    pub type_ref: usize,
    pub generics: Vec<TypeInfo>,
    pub array_dim: usize,
}

#[derive(Clone, Eq, PartialEq)]
pub struct GenericTypeInfo {
    pub parent_type: usize,
    pub generics: Vec<TypeInfo>,
    pub generic_ref: usize,
}

#[derive(Clone, Eq, PartialEq)]
pub enum TypeInfo {
    Real(RealTypeInfo),
    Generic(GenericTypeInfo),
}