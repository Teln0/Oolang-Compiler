use crate::ast::ASTTypeInfo;
use crate::compiler::method_ref_manager::MethodRefMap;
use crate::compiler::type_info::{GenericTypeInfo, RealTypeInfo, TypeInfo};
use crate::compiler::{AbsolutePath, Compiler};
use std::collections::HashMap;

pub struct ClassTypeRefKind<'a> {
    pub super_class: Option<TypeInfo>,
    pub impls: Vec<usize>,

    pub is_abstract: bool,

    pub field_name_to_field_ref: HashMap<&'a str, usize>,
    pub method_name_to_method_ref_map: HashMap<&'a str, MethodRefMap>,
}

pub enum TypeRefKind<'a> {
    Class(ClassTypeRefKind<'a>),
}

impl<'a> TypeRefKind<'a> {
    pub fn unwrap_class_ref(&self) -> &ClassTypeRefKind<'a> {
        match self {
            TypeRefKind::Class(c) => c,
            _ => panic!("unwrapped class on non class type ref kind"),
        }
    }

    pub fn unwrap_class_ref_mut(&mut self) -> &mut ClassTypeRefKind<'a> {
        match self {
            TypeRefKind::Class(c) => c,
            _ => panic!("unwrapped class on non class type ref kind"),
        }
    }
}

pub struct FieldRefGenericBound {
    pub super_requirements: Vec<TypeInfo>,
    pub impl_requirements: Vec<TypeInfo>,
}

pub struct TypeRef<'a> {
    pub kind: TypeRefKind<'a>,
    pub absolute_path: AbsolutePath<'a>,
    pub visibility: AbsolutePath<'a>,

    pub generic_bounds: Vec<FieldRefGenericBound>,
    pub name_to_generic_bound: HashMap<&'a str, usize>,
}

impl<'a> TypeRef<'a> {
    pub fn new_class(
        absolute_path: AbsolutePath<'a>,
        visibility: AbsolutePath<'a>,
        is_abstract: bool,
    ) -> Self {
        TypeRef {
            kind: TypeRefKind::Class(ClassTypeRefKind {
                super_class: None,
                impls: vec![],
                is_abstract,
                field_name_to_field_ref: HashMap::new(),
                method_name_to_method_ref_map: HashMap::new(),
            }),
            absolute_path,
            visibility,
            generic_bounds: vec![],
            name_to_generic_bound: HashMap::new(),
        }
    }
}

pub struct TypeRefManager<'a> {
    pub type_refs: Vec<TypeRef<'a>>,
    pub name_to_type_ref: HashMap<&'a str, usize>,
}

#[derive(Copy, Clone)]
pub struct TypeRefResolvingContext<'a, 'b> {
    pub origin: &'b [&'a str],
    pub origin_type_ref: Option<usize>,
}

pub enum TypeRefAddResult {
    Duplicate,
    Ok,
}

pub enum TypeRefResolvingResult {
    None,
    Real(usize),
    Generic(usize, usize),
}

impl<'a> TypeRefManager<'a> {
    pub fn new() -> Self {
        TypeRefManager {
            type_refs: vec![],
            name_to_type_ref: HashMap::new(),
        }
    }

    pub fn add(&mut self, type_ref: TypeRef<'a>) -> TypeRefAddResult {
        let current_index = self.type_refs.len();
        let name = *type_ref.absolute_path.elements.last().unwrap();
        self.type_refs.push(type_ref);
        if self.name_to_type_ref.contains_key(name) {
            TypeRefAddResult::Duplicate
        } else {
            self.name_to_type_ref.insert(name, current_index);
            TypeRefAddResult::Ok
        }
    }

    pub fn resolve_type_ref(
        &self,
        context: TypeRefResolvingContext<'a, '_>,
        path: &[&str],
    ) -> TypeRefResolvingResult {
        // TODO : Check visibility

        if path.len() == 1 {
            // One element path, must be a local type
            if let Some(type_ref) = self.name_to_type_ref.get(path[0]) {
                TypeRefResolvingResult::Real(*type_ref)
            } else {
                if let Some(origin_type_ref) = context.origin_type_ref {
                    if let Some(generic_ref) = self.type_refs[origin_type_ref]
                        .name_to_generic_bound
                        .get(path[0])
                    {
                        TypeRefResolvingResult::Generic(origin_type_ref, *generic_ref)
                    } else {
                        TypeRefResolvingResult::None
                    }
                } else {
                    TypeRefResolvingResult::None
                }
            }
        } else {
            todo!("add absolute path resolving")
        }
    }

    pub fn resolve_type_info_unchecked(
        &self,
        context: TypeRefResolvingContext<'a, '_>,
        type_info: &ASTTypeInfo,
    ) -> Option<TypeInfo> {
        match self.resolve_type_ref(context, &type_info.path.elements) {
            TypeRefResolvingResult::None => None,
            TypeRefResolvingResult::Real(type_ref) => {
                let mut generics = vec![];
                for generic in &type_info.generics {
                    let generic = self.resolve_type_info_unchecked(context, generic)?;
                    generics.push(generic);
                }
                Some(TypeInfo::Real(RealTypeInfo {
                    type_ref,
                    generics,
                    array_dim: type_info.array_dim,
                }))
            }
            TypeRefResolvingResult::Generic(parent_type, generic_ref) => {
                let mut generics = vec![];
                for generic in &type_info.generics {
                    let generic = self.resolve_type_info_unchecked(context, generic)?;
                    generics.push(generic);
                }
                Some(TypeInfo::Generic(GenericTypeInfo {
                    generic_ref,
                    generics,
                    parent_type,
                }))
            }
        }
    }

    pub fn resolve_type_info_checked(
        &self,
        context: TypeRefResolvingContext<'a, '_>,
        type_info: &ASTTypeInfo,
    ) -> Option<TypeInfo> {
        let type_info = self.resolve_type_info_unchecked(context, type_info)?;
        match type_info.check_generic_bounds(self) {
            Ok(_) => Some(type_info),
            Err(_) => None,
        }
    }
}