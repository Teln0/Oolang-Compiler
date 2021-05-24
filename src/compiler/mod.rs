pub mod cycle_detection;
pub mod ref_managers;

use crate::ast::{ASTRoot, ASTTypeKind, ASTModifier, ASTVisibility, ASTTypeInfo};
use crate::compiler::ref_managers::{FieldRefManager, MethodRefManager, TypeRefManager, AbsolutePath, TypeRef, TypeRefKind, TypeRefAddResult, TypeRefResolvingContext, TypeRefResolvingResult, GenericBound, TypeInfo};
use std::collections::HashMap;

pub struct Compiler<'a> {
    field_ref_manager: FieldRefManager<'a>,
    method_ref_manager: MethodRefManager<'a>,
    type_ref_manager: TypeRefManager<'a>
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Compiler {
            field_ref_manager: FieldRefManager::new(),
            method_ref_manager: MethodRefManager::new(),
            type_ref_manager: TypeRefManager::new()
        }
    }

    fn register_type_declarations(&mut self, root: &ASTRoot<'a>) {
        for type_decl in &root.types {
            match &type_decl.kind {
                ASTTypeKind::Class { .. } => {
                    let mut absolute_path = AbsolutePath::from_ast_path(&root.mod_decl.path);
                    absolute_path.elements.push(type_decl.name);
                    let mut is_abstract: bool = false;
                    for modifier in &type_decl.modifiers {
                        match modifier {
                            ASTModifier::Static => todo!("invalid modifier"),
                            ASTModifier::Abstract => if is_abstract { todo!("duplicate modifier") } else { is_abstract = true}
                            ASTModifier::Native => todo!("invalid modifier")
                        }
                    }
                    let visibility = match type_decl.visibility {
                        ASTVisibility::Public => AbsolutePath::empty(),
                        ASTVisibility::Module => AbsolutePath::from_ast_path(&root.mod_decl.path),
                        ASTVisibility::Private => absolute_path.clone()
                    };

                    match self.type_ref_manager.add(TypeRef::new_class(
                        absolute_path,
                        visibility,
                        is_abstract
                    )) {
                        TypeRefAddResult::Duplicate => todo!("duplicate"),
                        TypeRefAddResult::Ok => {}
                    }
                }
                _ => todo!()
            }
        }
    }

    fn register_type_generics(&mut self, root: &ASTRoot<'a>) {
        for type_decl in &root.types {
            let mut context = TypeRefResolvingContext {
                origin: &root.mod_decl.path.elements,
                origin_type_ref: None
            };
            let type_ref = match self.type_ref_manager.resolve_type_ref(context, &[type_decl.name]) {
                TypeRefResolvingResult::Real(type_ref) => type_ref,
                _ => unreachable!()
            };
            context.origin_type_ref = Some(type_ref);
            for generic_bound in &type_decl.generics {
                let name = generic_bound.name;
                let mut super_requirements = vec![];
                for requirement in &generic_bound.super_requirements {
                    super_requirements.push(if let Some(type_info) = self.type_ref_manager.resolve_type_info(context, &requirement.into_type_info()) {
                        type_info
                    }
                    else {
                        todo!("unknown")
                    });
                }
                let mut impl_requirements = vec![];
                for requirement in &generic_bound.impl_requirements {
                    impl_requirements.push(if let Some(type_info) = self.type_ref_manager.resolve_type_info(context, &requirement.into_type_info()) {
                        type_info
                    }
                    else {
                        todo!("unknown")
                    });
                }
                let generic_bound = GenericBound {
                    super_requirements,
                    impl_requirements
                };
                let generic_bounds = &mut self.type_ref_manager.type_refs[type_ref].generic_bounds;
                let current_index = generic_bounds.len();
                generic_bounds.push(generic_bound);
                let name_to_generic_bound = &mut self.type_ref_manager.type_refs[type_ref].name_to_generic_bound;
                if name_to_generic_bound.contains_key(name) {
                    todo!("duplicate")
                }
                name_to_generic_bound.insert(name, current_index);
            }
        }
    }

    fn register_type_extensions_and_implementations(&mut self, root: &ASTRoot<'a>) {
        for type_decl in &root.types {
            match &type_decl.kind {
                ASTTypeKind::Class { super_class, impls, .. } => {
                    if let Some(super_class) = super_class {
                        let mut context = TypeRefResolvingContext {
                            origin: &root.mod_decl.path.elements,
                            origin_type_ref: None
                        };
                        let type_ref = match self.type_ref_manager.resolve_type_ref(context, &[type_decl.name]) {
                            TypeRefResolvingResult::Real(type_ref) => type_ref,
                            _ => unreachable!()
                        };
                        context.origin_type_ref = Some(type_ref);
                        if let Some(super_class) = self.type_ref_manager.resolve_type_info(context, &super_class.into_type_info()) {
                            if let TypeInfo::Real { type_ref: super_class, .. } = super_class {
                                match &mut self.type_ref_manager.type_refs[type_ref].kind {
                                    TypeRefKind::Class { super_class: sc, .. } => {
                                        sc.replace(self.type_ref_manager.type_refs[super_class].clone());
                                    }
                                    _ => unreachable!()
                                }
                            }
                            else {
                                todo!("cannot extend generic")
                            }
                        }
                        else {
                            todo!("unknown")
                        };
                    }
                    for _ in impls {
                        unimplemented!()
                    }
                }
                _ => todo!()
            }
        }
    }

    pub fn validate_generics_extensions_and_implementations(&mut self, root: &ASTRoot<'a>) {
        for type_decl in &root.types {
            match &type_decl.kind {
                ASTTypeKind::Class { .. } => {
                    let mut context = TypeRefResolvingContext {
                        origin: &root.mod_decl.path.elements,
                        origin_type_ref: None
                    };
                    let type_ref = match self.type_ref_manager.resolve_type_ref(context, &[type_decl.name]) {
                        TypeRefResolvingResult::Real(type_ref) => type_ref,
                        _ => unreachable!()
                    };
                    context.origin_type_ref = Some(type_ref);

                    // Making sure there are no cycles in the super class

                }
                _ => todo!()
            }
        }
    }

    pub fn compile(mut self, root: &ASTRoot<'a>) {
        // TODO : process use statements

        self.register_type_declarations(root);
        self.register_type_generics(root);
        self.register_type_extensions_and_implementations(root);
        self.validate_generics_extensions_and_implementations(root);
    }
}