use oolang::lexer::Lexer;
use oolang::parser::Parser;
use oolang::reporting::ast_dumper::dump_from_root;

fn main() {
    let tokens = Lexer::new(
        "
    mod telno::testing;

    class A<T>
     where T <: SuperClassOfT<B> : InterfaceImplementedByT<C> {
    }
    ",
    )
    .lex()
    .unwrap();

    let ast = Parser::new(tokens).parse().unwrap();

    dump_from_root(&ast);
}
