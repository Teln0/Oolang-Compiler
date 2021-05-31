use crate::compiler::{CompiledType, CResult};
use crate::ast::{ASTType, ASTTypeInfo};
use crate::compiler::ty::{Ty, TyDef};

pub trait QueryExecutor<'a> {
    fn check_assignable(&self, to_assign: &Ty, assign_to: &Ty) -> CResult<()>;
    fn check_generics(&self, ty: &Ty, inside_generic: bool) -> CResult<()>;

    fn get_ty_def_of_ty(&self, ty: &Ty) -> &TyDef;
    fn get_ty_def_of_ast_ty(&self, ty: &ASTType) -> &TyDef;
    fn get_ty_unchecked(&self, type_info: &ASTTypeInfo<'a>) -> CResult<Ty>;

    fn compile_type(&self, type_def: &ASTType<'a>) -> CResult<CompiledType>;
    fn compile_all(&self) -> CResult<Vec<CompiledType>>;
}