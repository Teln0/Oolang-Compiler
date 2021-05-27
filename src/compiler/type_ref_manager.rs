use crate::ast::{ASTTypeInfo};
use std::collections::HashMap;
use crate::compiler::AbsolutePath;
use crate::compiler::method_ref_manager::{MethodRefMap};

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
    pub fn unwrap_class(&self) -> &ClassTypeRefKind<'a> {
        match self {
            TypeRefKind::Class(c) => c,
            _ => panic!("unwrapped class on non class type ref kind"),
        }
    }

    pub fn unwrap_class_mut(&mut self) -> &mut ClassTypeRefKind<'a> {
        match self {
            TypeRefKind::Class(c) => c,
            _ => panic!("unwrapped class on non class type ref kind"),
        }
    }
}

pub struct GenericBound {
    pub super_requirements: Vec<TypeInfo>,
    pub impl_requirements: Vec<TypeInfo>,
}

pub struct TypeRef<'a> {
    pub kind: TypeRefKind<'a>,
    pub absolute_path: AbsolutePath<'a>,
    pub visibility: AbsolutePath<'a>,

    pub generic_bounds: Vec<GenericBound>,
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

pub enum TypeRefAddResult {
    Duplicate,
    Ok,
}

#[derive(Copy, Clone)]
pub struct TypeRefResolvingContext<'a, 'b> {
    pub origin: &'b [&'a str],
    pub origin_type_ref: Option<usize>,
}

pub enum TypeRefResolvingResult {
    None,
    Real(usize),
    Generic(usize, usize),
}

pub enum GenericBoundsCheckingResult {
    Ok,
    LenMismatch,
    BoundErr,
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

    pub fn resolve_type_info(
        &self,
        context: TypeRefResolvingContext<'a, '_>,
        type_info: &ASTTypeInfo,
    ) -> Option<TypeInfo> {
        match self.resolve_type_ref(context, &type_info.path.elements) {
            TypeRefResolvingResult::None => None,
            TypeRefResolvingResult::Real(type_ref) => {
                let mut generics = vec![];
                for generic in &type_info.generics {
                    let generic = self.resolve_type_info(context, generic)?;
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
                    let generic = self.resolve_type_info(context, generic)?;
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

    pub fn get_super_class_of_real(&self, type_ref: usize) -> Option<RealTypeInfo> {
        let super_class = &self.type_refs[type_ref]
            .kind
            .unwrap_class()
            .super_class
            .clone();
        let super_class = super_class.clone()?;
        if let TypeInfo::Real(super_class) = super_class {
            Some(super_class)
        } else {
            None
        }
    }

    pub fn inherits(&self, type_info: &TypeInfo, supposed_super: &TypeInfo) -> bool {
        match type_info {
            TypeInfo::Real(RealTypeInfo { type_ref, .. }) => {
                match &self.type_refs[*type_ref].kind {
                    TypeRefKind::Class(ClassTypeRefKind { super_class, .. }) => {
                        if let Some(super_class) = super_class {
                            super_class == supposed_super
                                || self.inherits(super_class, supposed_super)
                        } else {
                            false
                        }
                    }
                    _ => todo!(),
                }
            }
            TypeInfo::Generic(GenericTypeInfo {
                parent_type,
                generic_ref,
                ..
            }) => {
                for requirement in
                    &self.type_refs[*parent_type].generic_bounds[*generic_ref].super_requirements
                {
                    // Check if any of the super requirements are or contain the supposed super class
                    if requirement == supposed_super || self.inherits(requirement, supposed_super) {
                        return true;
                    }
                }

                false
            }
        }
    }

    pub fn check_generic_bounds(&self, type_info: &TypeInfo) -> GenericBoundsCheckingResult {
        match type_info {
            TypeInfo::Real(RealTypeInfo {
                type_ref, generics, ..
            }) => {
                let generic_bounds = &self.type_refs[*type_ref].generic_bounds;
                if generic_bounds.len() != generics.len() {
                    return GenericBoundsCheckingResult::LenMismatch;
                }

                for i in 0..generic_bounds.len() {
                    if generic_bounds[i].impl_requirements.len() != 0 {
                        todo!()
                    }

                    for super_requirement in &generic_bounds[i].super_requirements {
                        if !self.inherits(&generics[i], super_requirement) {
                            return GenericBoundsCheckingResult::BoundErr;
                        }
                    }
                }

                GenericBoundsCheckingResult::Ok
            }
            TypeInfo::Generic(_) => todo!(),
        }
    }
}