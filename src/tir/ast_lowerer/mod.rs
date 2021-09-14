use crate::ast::{ASTRoot, ASTTypeKind, ASTTypeInfo, ASTModifier};
use crate::tir::{TIRRoot, TIRTypeInfo, TIRTypeInfoKind, TIRType, TIRTypeKind, PrimitiveType};
use crate::type_ref_pool::{TypeRefPool, TypeRef, TypeRefKind, ClassTypeRef, TypeRefGeneric};
use std::collections::HashMap;
use crate::tir::TIRTypeInfoKind::Primitive;

pub mod member_lowerer;

pub struct GenericContext<'a, 'b> {
    type_ref_index: usize,
    name_to_generic_index: &'b HashMap<&'a str, usize>
}

pub struct ASTtoTIRLowerer<'a> {
    mod_context: Vec<&'a str>,
    ast_root: ASTRoot<'a>,

    type_ref_pool: TypeRefPool<'a>
}

impl<'a> ASTtoTIRLowerer<'a> {
    pub fn new(ast_root: ASTRoot<'a>) -> Self {
        Self {
            mod_context: ast_root.mod_decl.path.elements.clone(),
            ast_root,

            type_ref_pool: TypeRefPool::new()
        }
    }

    fn check_generics(&self, type_info: &TIRTypeInfo) -> Result<(), ASTtoTIRLowererError<'a>> {
        match &type_info.kind {
            TIRTypeInfoKind::TypeRef { generics, type_ref_index, .. } => {
                let type_ref_index = *type_ref_index;
                let type_ref_generics = &self.type_ref_pool.type_refs[type_ref_index].generics;
                if generics.len() != type_ref_generics.len() {
                    return Err(ASTtoTIRLowererError::MismatchedGenerics(type_ref_index, generics.len(), type_ref_generics.len()));
                }
                for i in 0..generics.len() {
                    // We recursively check for nested generics
                    self.check_generics(&generics[i])?;
                    for type_reg_generic_requirement in &type_ref_generics[i].super_requirements {
                        if !self.type_ref_pool.check_assignable_to(&generics[i], type_reg_generic_requirement) {
                            return Err(ASTtoTIRLowererError::TypeMismatch)
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn resolve_type_info(&self, type_info: &ASTTypeInfo<'a>, generic_context: Option<&GenericContext<'a, '_>>) -> Result<TIRTypeInfo, ASTtoTIRLowererError<'a>> {
        if type_info.path.elements.len() == 1 {
            'primitive_test: loop {
                let primitive = match type_info.path.elements[0] {
                    "void" => PrimitiveType::Void,
                    "i64" => PrimitiveType::I64,
                    "i32" => PrimitiveType::I32,
                    "i16" => PrimitiveType::I16,
                    "i8" => PrimitiveType::I8,
                    "u64" => PrimitiveType::U64,
                    "u32" => PrimitiveType::U32,
                    "u16" => PrimitiveType::U16,
                    "u8" => PrimitiveType::U8,
                    "F64" => PrimitiveType::F64,
                    "F32" => PrimitiveType::F32,
                    "bool" => PrimitiveType::Boolean,
                    "char" => PrimitiveType::Character,
                    _ => break 'primitive_test
                };
                if !type_info.generics.is_empty() {
                    return Err(ASTtoTIRLowererError::GenericOnPrimitive);
                }
                return Ok(TIRTypeInfo {
                    kind: TIRTypeInfoKind::Primitive {
                        primitive,
                        array_dim: type_info.array_dim
                    },
                    span: type_info.span
                })
            }

            if let Some(g) = generic_context {
                if let Some(generic_index) = g.name_to_generic_index.get(type_info.path.elements[0]) {
                    if !type_info.generics.is_empty() {
                        return Err(ASTtoTIRLowererError::GenericOnGeneric(type_info.path.elements[0]));
                    }
                    return Ok(TIRTypeInfo {
                        kind: TIRTypeInfoKind::Generic {
                            type_ref_index: g.type_ref_index,
                            generic_index: *generic_index,
                            array_dim: type_info.array_dim
                        },
                        span: type_info.span
                    })
                }
            }
        }
        let type_ref_index = self.resolve_type_ref_index(&type_info.path.elements)?;
        Ok(TIRTypeInfo {
            kind: TIRTypeInfoKind::TypeRef {
                type_ref_index,
                generics: type_info.generics.iter().map(|g| { self.resolve_type_info(g, generic_context) }).collect::<Result<Vec<TIRTypeInfo>, ASTtoTIRLowererError<'a>>>()?,
                array_dim: type_info.array_dim
            },
            span: type_info.span
        })
    }

    fn resolve_type_ref_index(&self, path: &[&'a str]) -> Result<usize, ASTtoTIRLowererError<'a>> {
        // TODO : Check visibility
        if let Some(index) = self.type_ref_pool.full_path_to_type_ref_index.get(path) {
            Ok(*index)
        }
        else {
            let mut combined = self.mod_context.clone();
            combined.extend_from_slice(path);
            if let Some(index) = self.type_ref_pool.full_path_to_type_ref_index.get(&combined) {
                Ok(*index)
            }
            else {
                Err(ASTtoTIRLowererError::NoSuchType(path.to_vec()))
            }
        }
    }

    fn name_to_full_path(&self, name: &'a str) -> Vec<&'a str> {
        let mut result = self.mod_context.clone();
        result.push(name);
        result
    }

    #[inline(always)]
    fn register_types(&mut self) -> Result<(), ASTtoTIRLowererError<'a>> {
        // TODO : check if type has the same name as primitive (not allowed)

        for type_decl_index in 0..self.ast_root.types.len() {
            let type_decl = &self.ast_root.types[type_decl_index];
            let full_path = self.name_to_full_path(type_decl.name);
            let type_ref_index = self.type_ref_pool.type_refs.len();

            self.type_ref_pool.type_decl_index_to_type_ref_index.insert(type_decl_index, type_ref_index);
            match type_decl.kind {
                ASTTypeKind::Class { .. } => {
                    if let Some(_) = self.type_ref_pool.full_path_to_type_ref_index.insert(full_path.clone(), type_ref_index) {
                        return Err(ASTtoTIRLowererError::DuplicateTypeDecl(full_path.clone()));
                    }

                    let mut is_abstract = false;
                    for modifier in &type_decl.modifiers {
                        match modifier {
                            ASTModifier::Static => return Err(ASTtoTIRLowererError::ModifierNotCompatibleForClass(ASTModifier::Static)),
                            ASTModifier::Abstract => if is_abstract {
                                return Err(ASTtoTIRLowererError::DuplicateModifier(ASTModifier::Abstract));
                            } else {
                                is_abstract = true
                            },
                            ASTModifier::Native => return Err(ASTtoTIRLowererError::ModifierNotCompatibleForClass(ASTModifier::Native)),
                        }
                    }

                    self.type_ref_pool.type_refs.push(TypeRef {
                        kind: TypeRefKind::Class(ClassTypeRef {
                            // Will be filled in later (register_supers)
                            super_class: None,
                            is_abstract
                        }),
                        full_path,
                        // Will be filled in later (register_generics_boundless)
                        generics: vec![],
                        name_to_generic_index: HashMap::new()
                    });
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn register_generics_boundless(&mut self) -> Result<(), ASTtoTIRLowererError<'a>> {
        for type_decl_index in 0..self.ast_root.types.len() {
            let type_decl = &self.ast_root.types[type_decl_index];
            let type_ref_index = self.type_ref_pool.type_decl_index_to_type_ref_index[&type_decl_index];

            for generic in &type_decl.generics {
                let type_ref = &mut self.type_ref_pool.type_refs[type_ref_index];
                let generic_index = type_ref.generics.len();
                if let Some(_) = type_ref.name_to_generic_index.insert(generic.name, generic_index) {
                    let full_path = self.name_to_full_path(type_decl.name);
                    return Err(ASTtoTIRLowererError::DuplicateGeneric(full_path, generic.name))
                }
                type_ref.generics.push(TypeRefGeneric {
                    name: generic.name,
                    // Will be filled in later
                    super_requirements: vec![]
                })
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn register_supers(&mut self) -> Result<(), ASTtoTIRLowererError<'a>> {
        for type_decl_index in 0..self.ast_root.types.len() {
            let type_decl = &self.ast_root.types[type_decl_index];
            let type_ref_index = self.type_ref_pool.type_decl_index_to_type_ref_index[&type_decl_index];

            match &type_decl.kind {
                ASTTypeKind::Class { super_class, .. } => {
                    if let Some(super_class) = super_class {
                        let type_info = super_class.into_type_info();
                        let type_info = self.resolve_type_info(&type_info, None)?;
                        match type_info.kind {
                            TIRTypeInfoKind::TypeRef { .. } => {}
                            _ => return Err(ASTtoTIRLowererError::InvalidSuperClass(type_decl.name))
                        }
                        match &mut self.type_ref_pool.type_refs[type_ref_index].kind {
                            TypeRefKind::Class(class_type_ref) => {
                                class_type_ref.super_class = Some(type_info);
                            },
                            _ => unreachable!()
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn register_generic_bounds(&mut self) -> Result<(), ASTtoTIRLowererError<'a>> {
        for type_decl_index in 0..self.ast_root.types.len() {
            let type_decl = &self.ast_root.types[type_decl_index];
            let type_ref_index = self.type_ref_pool.type_decl_index_to_type_ref_index[&type_decl_index];

            for generic_index in 0..type_decl.generics.len() {
                let ast_generic = &type_decl.generics[generic_index];
                for requirement in &ast_generic.super_requirements {
                    let type_info = requirement.into_type_info();
                    let type_ref = &self.type_ref_pool.type_refs[type_ref_index];
                    let type_info = self.resolve_type_info(&type_info, Some(&GenericContext {
                        type_ref_index,
                        name_to_generic_index: &type_ref.name_to_generic_index
                    }))?;

                    let type_ref = &mut self.type_ref_pool.type_refs[type_ref_index];
                    // TODO : check for duplicates
                    type_ref.generics[generic_index].super_requirements.push(type_info);
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn lower_ast_root(&self) -> Result<TIRRoot<'a>, ASTtoTIRLowererError<'a>> {
        // TODO check for cycles in supers & generic super requirements
        // TODO check for generic super requirements having multiple super classes (only multiple interface impls are allowed)

        let mut types = vec![];

        for type_decl_index in 0..self.ast_root.types.len() {
            let type_decl = &self.ast_root.types[type_decl_index];
            let type_ref_index = self.type_ref_pool.type_decl_index_to_type_ref_index[&type_decl_index];
            let type_ref = &self.type_ref_pool.type_refs[type_ref_index];

            // Checking generics
            for generic in &self.type_ref_pool.type_refs[type_ref_index].generics {
                for generic_requirement in &generic.super_requirements {
                    self.check_generics(generic_requirement)?;
                }
            }

            match &self.type_ref_pool.type_refs[type_ref_index].kind {
                TypeRefKind::Class(class_type_ref) => {
                    // Checking superclass
                    let super_class = if let Some(super_class) = &class_type_ref.super_class {
                        self.check_generics(super_class)?;
                        Some(super_class.clone())
                    }
                    else {
                        None
                    };

                    // Lowering members
                    let mut lowered_members = vec![];
                    match &type_decl.kind {
                        ASTTypeKind::Class { members, .. } => {
                            for member in members {
                                lowered_members.push(self.lower_ast_member(member, &GenericContext {
                                    type_ref_index,
                                    name_to_generic_index: &type_ref.name_to_generic_index
                                })?);
                            }
                        }
                        _ => unreachable!()
                    }

                    types.push(TIRType {
                        type_ref_index,
                        kind: TIRTypeKind::Class {
                            members: lowered_members,
                            super_class
                        },
                        span: type_decl.span.clone()
                    })
                }
            }
        }

        Ok(TIRRoot {
            types,
            span: self.ast_root.span.clone()
        })
    }

    pub fn lower(mut self) -> Result<(TIRRoot<'a>, TypeRefPool<'a>), ASTtoTIRLowererError<'a>> {
        self.register_types()?;
        self.register_generics_boundless()?;
        self.register_supers()?;
        self.register_generic_bounds()?;
       Ok((self.lower_ast_root()?, self.type_ref_pool))
    }
}

#[derive(Debug)]
pub enum ASTtoTIRLowererError<'a> {
    DuplicateTypeDecl(Vec<&'a str>),
    DuplicateGeneric(Vec<&'a str>, &'a str),
    NoSuchType(Vec<&'a str>),
    GenericOnGeneric(&'a str),
    MismatchedGenerics(usize, usize, usize),
    GenericOnPrimitive,
    TypeMismatch,
    ModifierNotCompatibleForClass(ASTModifier),
    DuplicateModifier(ASTModifier),
    InvalidSuperClass(&'a str)
}