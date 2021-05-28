use crate::compiler::type_ref_manager::{ClassTypeRefKind, TypeRefKind, TypeRefManager};

#[derive(Clone, Eq, PartialEq)]
pub struct RealTypeInfo {
    pub type_ref: usize,
    pub generics: Vec<TypeInfo>,
    pub array_dim: usize,
}

impl RealTypeInfo {
    pub fn get_super_class(
        type_ref: usize,
        type_ref_manager: &TypeRefManager,
    ) -> Option<RealTypeInfo> {
        let super_class = type_ref_manager.type_refs[type_ref]
            .kind
            .unwrap_class_ref()
            .super_class
            .clone()?;
        Some(super_class.unwrap_real())
    }
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

pub enum GenericBoundCheckError {
    LenMismatch,
    BoundError,
}

impl TypeInfo {
    pub fn inherits(&self, supposed_super: &TypeInfo, type_ref_manager: &TypeRefManager) -> bool {
        self == supposed_super
            || match self {
                TypeInfo::Real(RealTypeInfo { type_ref, .. }) => {
                    match &type_ref_manager.type_refs[*type_ref].kind {
                        TypeRefKind::Class(ClassTypeRefKind { super_class, .. }) => {
                            if let Some(super_class) = super_class {
                                super_class == supposed_super
                                    || super_class.inherits(supposed_super, type_ref_manager)
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
                    for requirement in &type_ref_manager.type_refs[*parent_type].generic_bounds
                        [*generic_ref]
                        .super_requirements
                    {
                        // Check if any of the super requirements are or contain the supposed super class
                        if requirement == supposed_super
                            || requirement.inherits(supposed_super, type_ref_manager)
                        {
                            return true;
                        }
                    }

                    false
                }
            }
    }

    pub fn check_generic_bounds(
        &self,
        type_ref_manager: &TypeRefManager,
    ) -> Result<(), GenericBoundCheckError> {
        match self {
            TypeInfo::Real(RealTypeInfo {
                type_ref, generics, ..
            }) => {
                let generic_bounds = &type_ref_manager.type_refs[*type_ref].generic_bounds;
                if generic_bounds.len() != generics.len() {
                    return Err(GenericBoundCheckError::LenMismatch);
                }

                for i in 0..generic_bounds.len() {
                    if generic_bounds[i].impl_requirements.len() != 0 {
                        todo!()
                    }

                    for super_requirement in &generic_bounds[i].super_requirements {
                        if !generics[i].inherits(super_requirement, type_ref_manager) {
                            return Err(GenericBoundCheckError::BoundError);
                        }
                    }
                }

                Ok(())
            }
            TypeInfo::Generic(_) => todo!(),
        }
    }

    pub fn unwrap_real_ref(&self) -> &RealTypeInfo {
        match self {
            TypeInfo::Real(r) => r,
            _ => panic!("unwrapped real on non real type info"),
        }
    }

    pub fn unwrap_real(self) -> RealTypeInfo {
        match self {
            TypeInfo::Real(r) => r,
            _ => panic!("unwrapped real on non real type info"),
        }
    }
}
