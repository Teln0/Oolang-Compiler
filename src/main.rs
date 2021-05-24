use autolang_compiler::lexer::Lexer;
use autolang_compiler::parser::Parser;
use autolang_compiler::compiler::Compiler;

fn main() {
    let tokens = Lexer::new("
    mod telno::testing;

    class A<T> {
    }

    pub class Main<T: A<Main>>: A {
        pub static field_1: u64 = 5 + 6 + 3 + 7;

        pub static fn main(args: String[]) -> void {
            let a: str = \"test\";
        }

        pub abstract fn test();
    }
    ").lex().unwrap();

    let ast = Parser::new(tokens).parse().unwrap();

    Compiler::new().compile(&ast);
}
