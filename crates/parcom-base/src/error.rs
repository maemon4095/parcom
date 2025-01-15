use parcom_core::ParseError;

#[derive(Debug)]
pub struct Fatal<E>(pub E);

impl<E> ParseError for Fatal<E> {
    fn should_terminate(&self) -> bool {
        true
    }
}

impl<E> From<E> for Fatal<E> {
    fn from(value: E) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Miss<E>(pub E);

impl<E> ParseError for Miss<E> {
    fn should_terminate(&self) -> bool {
        false
    }
}

impl<E> From<E> for Miss<E> {
    fn from(value: E) -> Self {
        Self(value)
    }
}
