use oolang::lexer::Lexer;
use oolang::parser::Parser;
use oolang::tir::ast_lowerer::ASTtoTIRLowerer;
use oolang::codegen::Codegen;

fn main() {
    let tokens = Lexer::new(
        "
    mod telno::testing;

    class Main {
        field_1: Main = <telno::testing::Main>;

        pub static fn main(a: telno::testing::Main) {

        }
    }
    "
    )
    .lex()
    .unwrap();

    let ast = Parser::new(tokens).parse().unwrap();
    let (tir, type_ref_pool) = ASTtoTIRLowerer::new(ast).lower().unwrap();
    let bytecode_files = Codegen::new(tir, type_ref_pool).get_bytecode().unwrap();

    println!("{:?}", bytecode_files);
}
