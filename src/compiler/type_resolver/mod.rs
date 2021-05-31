use crate::compiler::ty::{TyDef, PrimitiveTyDef, ClassTyDef, GenericTyDef};
use std::collections::HashMap;
use crate::ast::*;
use crate::ast::visitor::{ASTVisitorMut, walk_type_default_mut};

pub struct TypeResolver<'a> {
    path_to_ty_def: HashMap<Vec<&'a str>, usize>,
    pub ty_defs: Vec<TyDef<'a>>,
    current_location: Vec<&'a str>
}

impl<'a> TypeResolver<'a> {
    pub fn new() -> Self {
        let mut path_to_ty_def = HashMap::new();
        path_to_ty_def.insert(vec!["u8"], 0);
        path_to_ty_def.insert(vec!["i8"], 1);
        path_to_ty_def.insert(vec!["u16"], 2);
        path_to_ty_def.insert(vec!["i16"], 3);
        path_to_ty_def.insert(vec!["u32"], 4);
        path_to_ty_def.insert(vec!["i32"], 5);
        path_to_ty_def.insert(vec!["u64"], 6);
        path_to_ty_def.insert(vec!["i64"], 7);
        path_to_ty_def.insert(vec!["u128"], 8);
        path_to_ty_def.insert(vec!["i128"], 9);

        path_to_ty_def.insert(vec!["f32"], 10);
        path_to_ty_def.insert(vec!["f64"], 11);

        path_to_ty_def.insert(vec!["bool"], 12);
        path_to_ty_def.insert(vec!["char"], 13);
        Self {
            ty_defs: vec![
                TyDef::Primitive(PrimitiveTyDef::U8),
                TyDef::Primitive(PrimitiveTyDef::I8),
                TyDef::Primitive(PrimitiveTyDef::U16),
                TyDef::Primitive(PrimitiveTyDef::I16),
                TyDef::Primitive(PrimitiveTyDef::U32),
                TyDef::Primitive(PrimitiveTyDef::I32),
                TyDef::Primitive(PrimitiveTyDef::U64),
                TyDef::Primitive(PrimitiveTyDef::I64),
                TyDef::Primitive(PrimitiveTyDef::U128),
                TyDef::Primitive(PrimitiveTyDef::I128),

                TyDef::Primitive(PrimitiveTyDef::F32),
                TyDef::Primitive(PrimitiveTyDef::F64),

                TyDef::Primitive(PrimitiveTyDef::Bool),
                TyDef::Primitive(PrimitiveTyDef::Char),
            ],
            path_to_ty_def,
            current_location: vec![]
        }
    }

    fn get_ty_def_for(&self, path: &[&'a str], current_location: &[&'a str]) -> Option<usize> {
        if self.path_to_ty_def.contains_key(path) {
            return Some(self.path_to_ty_def[path]);
        }

        // FIXME : could be more efficient
        let mut current_location = current_location.to_vec();
        while !current_location.is_empty() {
            let mut p = current_location.clone();
            p.append(&mut path.to_vec());

            if let Some(r) = self.path_to_ty_def.get(&p) {
                return Some(*r);
            }

            current_location.pop();
        }

        None
    }

    pub fn resolve_types(&mut self, root: &mut ASTRoot<'a>) {
        // FIXME : take "use" statements in account

        self.current_location = root.mod_decl.path.elements.clone();

        let path = root.mod_decl.path.elements.clone();
        for ty_decl in &mut root.types {
            let index = self.ty_defs.len();
            ty_decl.corresponding_type = index;
            let mut path = path.clone();
            path.push(ty_decl.name);

            if self.path_to_ty_def.contains_key(&path) { todo!("duplicate"); }
            else { self.path_to_ty_def.insert(path.clone(), index); }

            match &ty_decl.kind {
                ASTTypeKind::Class { .. } => {
                    self.ty_defs.push(TyDef::Class(ClassTyDef {
                        name: ty_decl.name,
                        super_class: None,
                        generics: Vec::with_capacity(ty_decl.generics.len())
                    }));

                    let mut generics = vec![];
                    for generic in &ty_decl.generics {
                        let generic_index = self.ty_defs.len();
                        generics.push(generic_index);
                        self.ty_defs.push(TyDef::Generic(GenericTyDef {
                            name: generic,
                            super_requirements: vec![],
                            impl_requirements: vec![],
                            belongs_to: index
                        }));

                        let mut path = path.clone();
                        path.push(generic);
                        if self.path_to_ty_def.contains_key(&path) { todo!("duplicate"); }
                        else { self.path_to_ty_def.insert(path, generic_index); }
                    }
                    self.ty_defs[index].unwrap_class_ref_mut().generics = generics;
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

        self.walk_root(root);
    }
}

impl<'a> ASTVisitorMut<'a> for TypeResolver<'a> {
    fn walk_type(&mut self, obj: &mut ASTType<'a>) {
        self.current_location.push(obj.name);
        walk_type_default_mut(self, obj);
        self.current_location.pop();
    }

    fn walk_path(&mut self, obj: &mut ASTPath) {
        let ty_def = self.get_ty_def_for(&obj.elements, &self.current_location);

        obj.corresponding_type = ty_def;
    }
}