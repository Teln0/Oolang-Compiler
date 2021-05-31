use crate::compiler::query::QueryExecutor;
use crate::compiler::ty::{TyDef, Ty, ClassTyDef, GenericTyDef};
use crate::ast::{ASTRoot, ASTType, ASTTypeKind, ASTTypeInfo};
use std::ops::Deref;

pub mod type_resolver;
pub mod query;
pub mod ty;
pub mod utils;

pub type CResult<T> = Result<T, CompilerError>;

pub struct Path<'a> {
    pub elements: Vec<&'a str>
}

pub struct NameScope<'a> {
    pub name: &'a str
}

pub struct GlobalCx<'a> {
    pub ast_root: ASTRoot<'a>,
    pub ty_defs: Vec<TyDef<'a>>
}

impl<'a> GlobalCx<'a> {
    pub fn register_modifiers_supers_and_impls(&mut self) -> CResult<()> {
        for type_def in &self.ast_root.types {
            match &type_def.kind {
                ASTTypeKind::Class { supers, .. } => {
                    if supers.len() > 1 {
                        todo!("cannot have more than one super class");
                    }
                    if !supers.is_empty() {
                        let super_ty = self.get_ty_unchecked(&supers[0].into_type_info())?;
                        self.ty_defs[type_def.corresponding_type].unwrap_class_ref_mut().super_class.replace(super_ty);
                    }

                    // TODO : register modifiers
                }
                ASTTypeKind::Inter { .. } => {
                    unimplemented!()
                }
                ASTTypeKind::Enum { .. } => {
                    unimplemented!()
                }
                ASTTypeKind::Impl { .. } => {
                    unimplemented!()
                }
            }
        }

        Ok(())
    }

    pub fn register_generic_bounds(&mut self) -> CResult<()> {
        for type_def in &self.ast_root.types {
            for generic_bound in &type_def.generic_bounds {
                let generic_type_def = if let Some(g) = generic_bound.path.corresponding_type { g }
                else { todo!("unresolved generic"); };
                for super_requirement in &generic_bound.super_requirements {
                    let ty = self.get_ty_unchecked(&super_requirement.into_type_info())?;
                    self.ty_defs[generic_type_def].unwrap_generic_ref_mut().super_requirements.push(ty);
                }
                for impl_requirement in &generic_bound.impl_requirements {
                    let ty = self.get_ty_unchecked(&impl_requirement.into_type_info());
                    let ty = if let Ok(ty) = ty { ty }
                    else { todo!("unresolved type def"); };
                    self.ty_defs[generic_type_def].unwrap_generic_ref_mut().impl_requirements.push(ty);
                }
            }
        }

        Ok(())
    }
}

pub struct TyCx<'a> {
    pub gcx: &'a GlobalCx<'a>
}

pub struct CompiledType<'a> {
    pub file_path: Path<'a>,
    pub file_data: Vec<u8>
}

impl<'a> QueryExecutor<'a> for GlobalCx<'a> {
    fn check_assignable(&self, to_assign: &Ty, assign_to: &Ty) -> CResult<()> {
        if to_assign == assign_to {
            return Ok(())
        }

        match self.get_ty_def_of_ty(assign_to) {
            TyDef::Class(_) => {
                match self.get_ty_def_of_ty(to_assign) {
                    TyDef::Class(
                        ClassTyDef {
                            super_class,
                            ..
                        }
                    ) => {
                        if let Some(super_class) = super_class {
                            return self.check_assignable(super_class, assign_to);
                        }

                        todo!("incompatible types")
                    }
                    TyDef::Primitive(_) => {
                        todo!("incompatible types")
                    }
                    TyDef::Generic(
                        GenericTyDef {
                            super_requirements,
                            ..
                        }
                    ) => {
                        for super_requirement in super_requirements {
                            if let Ok(_) = self.check_assignable(super_requirement, assign_to) {
                                return Ok(())
                            }
                        }
                        todo!("incompatible types")
                    }
                }
            }
            TyDef::Primitive(_) => {
                unimplemented!()
            }
            TyDef::Generic(
                GenericTyDef {
                    super_requirements,
                    impl_requirements,
                    ..
                }
            ) => {
                for super_requirement in super_requirements {
                    self.check_assignable(to_assign, super_requirement)?;
                }
                for impl_requirement in impl_requirements {
                    self.check_assignable(to_assign, impl_requirement)?;
                }

                Ok(())
            }
        }
    }

    fn check_generics(&self, ty: &Ty, primitives_allowed: bool) -> CResult<()> {
        match &self.ty_defs[ty.ty_def_id] {
            TyDef::Class(
                ClassTyDef {
                    generics,
                    ..
                }
            ) => {
                if ty.generics.len() != generics.len() {
                    todo!("generic len mismatch")
                }

                for i in 0..ty.generics.len() {
                    self.check_generics(&ty.generics[i], true)?;
                    self.check_assignable(&ty.generics[i], &Ty {
                        ty_def_id: generics[i],
                        generics: vec![],
                        array_dim: 0
                    })?;
                }

                Ok(())
            }
            TyDef::Primitive(_) => {
                if !ty.generics.is_empty() {
                    todo!("cannot have generics on primitives")
                }
                if primitives_allowed {
                    todo!("primitives not allowed in this context");
                }
                Ok(())
            }
            TyDef::Generic(_) => {
                if !ty.generics.is_empty() {
                    todo!("cannot have generics on generics")
                }
                Ok(())
            }
        }
    }

    fn get_ty_def_of_ty(&self, ty: &Ty) -> &TyDef {
        &self.ty_defs[ty.ty_def_id]
    }

    fn get_ty_def_of_ast_ty(&self, ty: &ASTType) -> &TyDef {
        &self.ty_defs[ty.corresponding_type]
    }

    fn get_ty_unchecked(&self, type_info: &ASTTypeInfo<'a>) -> CResult<Ty> {
        if let Some(ty_def_id) = type_info.path.corresponding_type {
            let ty = Ty {
                array_dim: type_info.array_dim,
                generics: type_info.generics.iter().map(|e| { self.get_ty_unchecked(e) }).collect::<CResult<Vec<Ty>>>()?,
                ty_def_id
            };

            Ok(ty)
        }
        else {
            todo!("unresolved type def")
        }
    }

    fn compile_type(&self, type_def: &ASTType<'a>) -> CResult<CompiledType> {
        match &type_def.kind {
            ASTTypeKind::Class { .. } => {
                let class_ref = self.get_ty_def_of_ast_ty(type_def).unwrap_class_ref();

                // Performing checks on generics ...
                if let Some(super_class) = &class_ref.super_class {
                    self.check_generics(super_class, false)?;
                    match self.get_ty_def_of_ty(super_class) {
                        TyDef::Class(_) => {},
                        _ => {
                            todo!("cannot have non class as super of class")
                        }
                    }
                }

                for i in &class_ref.generics {
                    match &self.ty_defs[*i] {
                        TyDef::Generic(
                            GenericTyDef {
                                super_requirements,
                                impl_requirements,
                                ..
                            }
                        ) => {
                            if super_requirements.len() > 1 {
                                todo!("cannot have more than one super on generic")
                            }

                            for ty in super_requirements {
                                self.check_generics(ty, false)?;
                                match self.get_ty_def_of_ty(ty) {
                                    TyDef::Class(_) => {}
                                    TyDef::Generic(_) => {}
                                    _ => {
                                        todo!("cannot have non class and non generic as super of generic")
                                    }
                                }
                            }

                            for _i in impl_requirements {
                                unimplemented!()
                            }
                        }
                        _ => todo!("must be generic")
                    }
                }
                // ... Generic checks done

                Ok(CompiledType {
                    file_path: Path {
                        elements: vec!["dummy"]
                    },
                    file_data: vec![]
                })
            }
            ASTTypeKind::Inter { .. } => {
                unimplemented!()
            }
            ASTTypeKind::Enum { .. } => {
                unimplemented!()
            }
            ASTTypeKind::Impl { .. } => {
                unimplemented!()
            }
        }
    }

    fn compile_all(&self) -> CResult<Vec<CompiledType>> {
        let mut result = vec![];

        for type_def in &self.ast_root.types {
            result.push(self.compile_type(type_def)?)
        }

        Ok(result)
    }
}

impl<'a> Deref for TyCx<'a> {
    type Target = GlobalCx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.gcx
    }
}

pub struct CompilerError {

}