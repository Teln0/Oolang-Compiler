use std::collections::HashSet;

pub fn check_cycles<T: Eq + Clone, FnNext: Fn(T) -> Option<T>>(starting_point: T, next: FnNext) -> bool {
    let mut tested = vec![starting_point.clone()];
    let mut current = starting_point;
    while let Some(some_current) = next(current) {
        if tested.contains(&some_current) {
            return false;
        }
        tested.push(some_current.clone());
        current = some_current;
    }

    true
}
