use crate::tir::TIRTypeInfo;
use std::collections::HashMap;

pub struct FieldRef<'a> {
    pub associated_type_ref_index: usize,

    pub type_info: TIRTypeInfo,
    pub name: &'a str,
    pub index: usize,
    pub index_in_all_members: usize,

    pub is_static: bool
}

pub struct FieldRefPool<'a> {
    pub field_refs: Vec<FieldRef<'a>>,
    pub type_ref_index_and_name_to_field_ref_index: HashMap<(usize, &'a str), usize>,
}

impl<'a> FieldRefPool<'a> {
    pub fn new() -> Self {
        Self {
            field_refs: vec![],
            type_ref_index_and_name_to_field_ref_index: HashMap::new()
        }
    }
}