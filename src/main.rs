use oolang::lexer::Lexer;
use oolang::parser::Parser;
use oolang::compiler::type_resolver::TypeResolver;
use oolang::compiler::{GlobalCx, TyCx};
use oolang::compiler::query::QueryExecutor;

fn main() {
    let tokens = Lexer::new(
        "
    mod telno::testing;

    class C {}

    class B<T>
        where T <: C {}

    class A<T, T2>
        where T <: B<T2>
        where T2 <: C {}
    ",
    )
    .lex()
    .unwrap();

    let mut ast_root = Parser::new(tokens).parse().unwrap();

    let mut type_resolver = TypeResolver::new();
    type_resolver.resolve_types(&mut ast_root);

    let mut global_context = GlobalCx {
        ty_defs: type_resolver.ty_defs,
        ast_root
    };

    if let Err(_) = global_context.register_generic_bounds() {
        panic!();
    }
    if let Err(_) = global_context.register_modifiers_supers_and_impls() {
        panic!();
    }

    let type_context = TyCx {
        gcx: &global_context
    };

    let _ = type_context.compile_all();
}
