use crate::ast::{ASTExpr, ASTMember, ASTNameAndType};
use crate::compiler::ref_managers::TypeRefResolvingContext;
use crate::compiler::Compiler;

impl<'a> Compiler<'a> {
    pub fn process_new_field(
        &mut self,
        parent_type_ref: usize,
        context: TypeRefResolvingContext,
        name_and_type: &ASTNameAndType<'a>,
        expression: &Option<ASTExpr<'a>>,
    ) {

    }
}
