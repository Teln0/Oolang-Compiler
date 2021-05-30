use crate::compiler::Path;

pub struct Ty<'a> {
    pub absolute_path: Path<'a>,
    pub generics: Vec<Ty<'a>>
}