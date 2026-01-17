pub trait Predicate<T> {
    fn test(&self, item: &T) -> bool;
}

impl<T, F: Fn(&T) -> bool> Predicate<T> for F {
    fn test(&self, item: &T) -> bool {
        (self)(item)
    }
}
