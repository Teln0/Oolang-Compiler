use crate::tir::TIRRoot;
use crate::type_ref_pool::TypeRefPool;
use crate::field_ref_pool::FieldRefPool;

pub struct BytecodeFile<'a> {
    pub file_path: Vec<&'a str>,
    pub bytecode: Vec<u8>
}

pub struct Codegen<'a> {
    tir_root: TIRRoot<'a>,

    type_ref_pool: TypeRefPool<'a>,
    field_ref_pool: FieldRefPool<'a>
}

impl<'a> Codegen<'a> {
    pub fn new(tir_root: TIRRoot<'a>, type_ref_pool: TypeRefPool<'a>) -> Self {
        Self {
            tir_root,
            type_ref_pool,
            field_ref_pool: FieldRefPool::new()
        }
    }

    fn create_field_and_method_refs(&mut self) -> Result<(), CodegenError> {
        todo!()
    }

    pub fn get_bytecode(mut self) -> Result<Vec<BytecodeFile<'a>>, CodegenError> {
        self.create_field_and_method_refs()?;

        todo!()
    }
}

#[derive(Debug)]
pub enum CodegenError {

}