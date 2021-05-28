use crate::compiler::AbsolutePath;
use std::collections::HashMap;

pub struct MethodRef<'a> {
    pub type_absolute_path: AbsolutePath<'a>,
    pub visibility: AbsolutePath<'a>,
    pub name: &'a str,

    pub is_static: bool,
    pub is_abstract: bool,
    pub is_native: bool,
}

pub struct MethodRefManager<'a> {
    pub method_refs: Vec<MethodRef<'a>>,
}

pub struct MethodRefMap {
    // Vectors of usize represent a list of parameters. Each usize is a type ref
    pub parameters_to_method_ref: HashMap<Vec<usize>, usize>,
}

impl<'a> MethodRefManager<'a> {
    pub fn new() -> Self {
        MethodRefManager {
            method_refs: vec![],
        }
    }
}
