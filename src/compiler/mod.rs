pub mod query;
pub mod ty;
pub mod utils;

pub struct Path<'a> {
    _elements: Vec<&'a str>
}

pub struct NameScope<'a> {
    _name: &'a str
}

pub struct GlobalCx {

}