pub trait Getter<T, U> {
    fn get(&self, value: T) -> U;
}

impl<T, U, F: Fn(T) -> U> Getter<T, U> for F {
    fn get(&self, value: T) -> U {
        (self)(value)
    }
}
