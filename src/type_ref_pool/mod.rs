use std::collections::HashMap;
use crate::tir::TIRTypeInfo;

#[derive(Debug)]
pub struct TypeRefGeneric<'a> {
    pub name: &'a str,
    pub super_requirements: Vec<TIRTypeInfo>,
}

#[derive(Debug)]
pub struct ClassTypeRef {
    pub super_class: Option<TIRTypeInfo>,

    pub is_abstract: bool
}

#[derive(Debug)]
pub enum TypeRefKind {
    Class(ClassTypeRef)
}

#[derive(Debug)]
pub struct TypeRef<'a> {
    pub full_path: Vec<&'a str>,
    pub kind: TypeRefKind,
    pub generics: Vec<TypeRefGeneric<'a>>,
    pub name_to_generic_index: HashMap<&'a str, usize>
}

#[derive(Debug)]
pub struct TypeRefPool<'a> {
    pub type_refs: Vec<TypeRef<'a>>,
    pub type_decl_index_to_type_ref_index: HashMap<usize, usize>,
    pub full_path_to_type_ref_index: HashMap<Vec<&'a str>, usize>
}

impl<'a> TypeRefPool<'a> {
    pub fn new() -> Self {
        Self {
            type_refs: vec![],
            type_decl_index_to_type_ref_index: HashMap::new(),
            full_path_to_type_ref_index: HashMap::new()
        }
    }

    pub fn check_assignable_to(&self, to_assign: &TIRTypeInfo, type_info: &TIRTypeInfo) -> bool {
        if to_assign == type_info {
            true
        }
        else {
            // TODO : Check super classes and assignment to generics
            todo!()
        }
    }
}