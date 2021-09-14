use std::collections::HashMap;
use crate::tir::TIRTypeInfo;

pub struct MethodRef<'a> {
    pub associated_type_ref_index: usize,

    pub return_type: TIRTypeInfo,
    pub name: &'a str,
    pub parameters: Vec<TIRTypeInfo>,
    pub index: usize,
    pub index_in_all_members: usize,

    pub is_abstract: bool,
    pub is_static: bool,
    pub is_native: bool
}

pub struct MethodRefPool<'a> {
    pub method_refs: Vec<MethodRef<'a>>,
    pub name_and_type_ref_index_to_method_ref_indexes: HashMap<(usize, &'a str), HashMap<Vec<TIRTypeInfo>, usize>>
}

impl<'a> MethodRefPool<'a> {
    pub fn new() -> Self {
        Self {
            method_refs: vec![],
            name_and_type_ref_index_to_method_ref_indexes: HashMap::new()
        }
    }
}