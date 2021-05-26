use oolang::compiler::Compiler;
use oolang::lexer::Lexer;
use oolang::parser::Parser;

fn main() {
    let tokens = Lexer::new(
        "
    mod telno::testing;

    class C {
    }

    class B<T: C> {
    }

    class Main<T: C, T2: B<T>> {

    }
    ",
    )
    .lex()
    .unwrap();

    let ast = Parser::new(tokens).parse().unwrap();

    Compiler::new().compile(&ast);
}
