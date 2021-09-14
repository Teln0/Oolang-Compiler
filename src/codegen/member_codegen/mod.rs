use crate::codegen::{Codegen, CodegenError};
use crate::tir::{TIRExpr, TIRTypeInfo};

impl<'a> Codegen<'a> {
    pub fn codegen_tir_expr(&self, tir_expr: &TIRExpr<'a>, bytecode: &mut Vec<u8>, codegen_context: &mut CodegenContext<'a>) -> Result<TIRTypeInfo, CodegenError<'a>> {
        todo!()
    }
}