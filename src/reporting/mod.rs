pub mod ast_dumper;
pub mod string_tree;

#[derive(Debug, Copy, Clone)]
pub struct CharSpan {
    pub base: usize,
    pub len: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct TokenSpan {
    pub base: usize,
    pub len: usize,
}

impl TokenSpan {
    pub fn new(base: usize, len: usize) -> Self {
        TokenSpan { base, len }
    }

    pub fn new_rn_ex(base: usize, end_exclusive: usize) -> Self {
        TokenSpan {
            base,
            len: end_exclusive - base,
        }
    }
}
