use std::collections::HashSet;

pub fn check_cycles<T: PartialEq, FnNext: Fn(T) -> Option<T>>(starting_point: T, next: FnNext) {
    let mut set = HashSet::new();
    set.insert(&starting_point);
}