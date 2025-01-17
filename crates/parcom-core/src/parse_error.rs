pub trait ParseError {
    fn should_terminate(&self) -> bool;
}

impl<T: ParseError> ParseError for &T {
    fn should_terminate(&self) -> bool {
        T::should_terminate(self)
    }
}

impl<T: ParseError> ParseError for Option<T> {
    fn should_terminate(&self) -> bool {
        match self {
            Some(v) => v.should_terminate(),
            None => false,
        }
    }
}
