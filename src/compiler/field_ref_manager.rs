use crate::compiler::type_info::TypeInfo;
use crate::compiler::type_ref_manager::{TypeRefKind, TypeRefManager};
use crate::compiler::AbsolutePath;

pub struct FieldRef<'a> {
    pub type_info: TypeInfo,
    pub visibility: AbsolutePath<'a>,
    pub name: &'a str,

    pub is_static: bool,
}

pub enum FieldRefAddResult {
    Duplicate,
    Ok,
}

pub struct FieldRefManager<'a> {
    pub field_refs: Vec<FieldRef<'a>>,
}

impl<'a> FieldRefManager<'a> {
    pub fn new() -> Self {
        FieldRefManager { field_refs: vec![] }
    }

    pub fn add(
        &mut self,
        field_ref: FieldRef<'a>,
        parent_type_ref: usize,
        type_ref_manager: &mut TypeRefManager<'a>,
    ) -> FieldRefAddResult {
        let current_index = self.field_refs.len();
        let name = field_ref.name;
        self.field_refs.push(field_ref);
        match &mut type_ref_manager.type_refs[parent_type_ref].kind {
            TypeRefKind::Class(class) => {
                if class.field_name_to_field_ref.contains_key(name) {
                    FieldRefAddResult::Duplicate
                } else {
                    class.field_name_to_field_ref.insert(name, current_index);
                    FieldRefAddResult::Ok
                }
            }
            _ => panic!("cannot add field to non class"),
        }
    }
}
