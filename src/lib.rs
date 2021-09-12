pub mod ast;
pub mod codegen;
// Stands for "typed intermediate representation" (basically an AST with full types filled in and useless data removed)
pub mod tir;
pub mod lexer;
pub mod parser;
pub mod reporting;

pub mod field_ref_pool;
pub mod type_ref_pool;
