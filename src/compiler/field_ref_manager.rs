use crate::compiler::AbsolutePath;
use crate::compiler::type_ref_manager::TypeRef;

pub struct FieldRef<'a> {
    pub type_absolute_path: AbsolutePath<'a>,
    pub visibility: AbsolutePath<'a>,
    pub name: &'a str,

    pub is_static: bool,
}

pub struct FieldRefManager<'a> {
    pub field_refs: Vec<TypeRef<'a>>,
}

impl<'a> FieldRefManager<'a> {
    pub fn new() -> Self {
        FieldRefManager { field_refs: vec![] }
    }
}