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
        field_1: B<C>;
    }
    ",
    )
    .lex()
    .unwrap();

    let _ast = Parser::new(tokens).parse().unwrap();
}
