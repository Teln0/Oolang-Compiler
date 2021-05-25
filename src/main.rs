use autolang_compiler::compiler::Compiler;
use autolang_compiler::lexer::Lexer;
use autolang_compiler::parser::Parser;

fn main() {
    let tokens = Lexer::new(
        "
    mod telno::testing;

    class C {}

    class B: C {}

    class A<T: C> {}

    class Main: A<B> {

    }
    ",
    )
    .lex()
    .unwrap();

    let ast = Parser::new(tokens).parse().unwrap();

    Compiler::new().compile(&ast);
}
