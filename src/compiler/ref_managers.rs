use std::collections::HashMap;
use crate::ast::{ASTPath, ASTTypeInfo};

#[derive(Clone)]
pub struct AbsolutePath<'a> {
    pub elements: Vec<&'a str>
}

impl<'a> AbsolutePath<'a> {
    pub fn empty() -> Self {
        AbsolutePath {
            elements: vec![]
        }
    }

    pub fn from_ast_path(path: &ASTPath<'a>) -> Self {
        AbsolutePath {
            elements: path.elements.clone()
        }
    }
}



pub struct MethodRef<'a> {
    pub type_absolute_path: AbsolutePath<'a>,
    pub visibility: AbsolutePath<'a>,
    pub name: &'a str,

    pub is_static: bool,
    pub is_abstract: bool,
    pub is_native: bool
}

pub struct FieldRef<'a> {
    pub type_absolute_path: AbsolutePath<'a>,
    pub visibility: AbsolutePath<'a>,
    pub name: &'a str,

    pub is_static: bool
}

pub enum TypeInfo {
    Real {
        type_ref: usize,
        generics: Vec<TypeInfo>,
        array_dim: usize
    },
    Generic {
        parent_type: usize,
        generics: Vec<TypeInfo>,
        generic_ref: usize
    }
}

pub enum TypeRefKind<'a> {
    Class {
        super_class: Option<TypeInfo>,
        impls: Vec<usize>,

        is_abstract: bool,

        field_name_to_field_ref: HashMap<&'a str, usize>,
        method_name_to_method_ref_map: HashMap<&'a str, MethodRefMap>
    }
}

#[derive(Clone)]
pub struct GenericBound {
    pub super_requirements: Vec<TypeInfo>,
    pub impl_requirements: Vec<TypeInfo>,
}

#[derive(Clone)]
pub struct TypeRef<'a> {
    pub kind: TypeRefKind<'a>,
    pub absolute_path: AbsolutePath<'a>,
    pub visibility: AbsolutePath<'a>,

    pub generic_bounds: Vec<GenericBound>,
    pub name_to_generic_bound: HashMap<&'a str, usize>
}

impl<'a> TypeRef<'a> {
    pub fn new_class(absolute_path: AbsolutePath<'a>, visibility: AbsolutePath<'a>, is_abstract: bool) -> Self {
        TypeRef {
            kind: TypeRefKind::Class {
                super_class: None,
                impls: vec![],
                is_abstract,
                field_name_to_field_ref: HashMap::new(),
                method_name_to_method_ref_map: HashMap::new()
            },
            absolute_path,
            visibility,
            generic_bounds: vec![],
            name_to_generic_bound: HashMap::new()
        }
    }
}



pub struct MethodRefManager<'a> {
    pub method_refs: Vec<MethodRef<'a>>
}

pub struct MethodRefMap {
    // Vectors of usize represent a list of parameters. Each usize is a type ref
    pub parameters_to_method_ref: HashMap<Vec<usize>, usize>
}

impl<'a> MethodRefManager<'a> {
    pub fn new() -> Self {
        MethodRefManager {
            method_refs: vec![]
        }
    }
}

pub struct FieldRefManager<'a> {
    pub field_refs: Vec<TypeRef<'a>>
}

impl<'a> FieldRefManager<'a> {
    pub fn new() -> Self {
        FieldRefManager {
            field_refs: vec![]
        }
    }
}

pub struct TypeRefManager<'a> {
    pub type_refs: Vec<TypeRef<'a>>,
    pub name_to_type_ref: HashMap<&'a str, usize>
}

pub enum TypeRefAddResult {
    Duplicate,
    Ok
}

#[derive(Copy, Clone)]
pub struct TypeRefResolvingContext<'a, 'b> {
    pub origin: &'b [&'a str],
    pub origin_type_ref: Option<usize>
}

pub enum TypeRefResolvingResult {
    None,
    Real(usize),
    Generic(usize, usize)
}

impl<'a> TypeRefManager<'a> {
    pub fn new() -> Self {
        TypeRefManager {
            type_refs: vec![],
            name_to_type_ref: HashMap::new()
        }
    }

    pub fn add(&mut self, type_ref: TypeRef<'a>) -> TypeRefAddResult {
        let current_index = self.type_refs.len();
        let name = *type_ref.absolute_path.elements.last().unwrap();
        self.type_refs.push(type_ref);
        if self.name_to_type_ref.contains_key(name) {
            TypeRefAddResult::Duplicate
        }
        else {
            self.name_to_type_ref.insert(name, current_index);
            TypeRefAddResult::Ok
        }
    }

    pub fn resolve_type_ref(&self, context: TypeRefResolvingContext<'a, '_>, path: &[&str]) -> TypeRefResolvingResult {
        // TODO : Check visibility

        if path.len() == 1 {
            // One element path, must be a local type
            if let Some(type_ref) = self.name_to_type_ref.get(path[0]) {
                TypeRefResolvingResult::Real(*type_ref)
            }
            else {
                if let Some(origin_type_ref) = context.origin_type_ref {
                    if let Some(generic_ref) = self.type_refs[origin_type_ref].name_to_generic_bound.get(path[0]) {
                        TypeRefResolvingResult::Generic(origin_type_ref, *generic_ref)
                    }
                    else {
                        TypeRefResolvingResult::None
                    }
                }
                else {
                    TypeRefResolvingResult::None
                }
            }
        }
        else {
            unimplemented!("absolute path resolving")
        }
    }

    pub fn resolve_type_info(&self, context: TypeRefResolvingContext<'a, '_>, type_info: &ASTTypeInfo) -> Option<TypeInfo> {
        match self.resolve_type_ref(context, &type_info.path.elements) {
            TypeRefResolvingResult::None => None,
            TypeRefResolvingResult::Real(type_ref) => {
                let mut generics = vec![];
                for generic in &type_info.generics {
                    let generic = self.resolve_type_info(context, generic)?;
                    generics.push(generic);
                }
                Some(TypeInfo::Real {
                    type_ref,
                    generics,
                    array_dim: type_info.array_dim
                })
            }
            TypeRefResolvingResult::Generic(parent_type, generic_ref) => {
                let mut generics = vec![];
                for generic in &type_info.generics {
                    let generic = self.resolve_type_info(context, generic)?;
                    generics.push(generic);
                }
                Some(TypeInfo::Generic {
                    generic_ref,
                    generics,
                    parent_type
                })
            }
        }
    }

    pub fn satisfies_generic_bound(type_info: &TypeInfo, generic_bound: &GenericBound) -> bool {
        unimplemented!()
    }
}