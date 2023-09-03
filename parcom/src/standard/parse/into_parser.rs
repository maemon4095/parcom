use std::marker::PhantomData;

use crate::{Parse, Parser};
pub struct IntoParser<T, P: Parse<T>>(pub(super) PhantomData<(T, P)>);

impl<T, P: Parse<T>> Parser<T> for IntoParser<T, P> {
    type Output = P;
    type Error = P::Error;

    fn parse(&self, input: T) -> parcom_core::ParseResult<T, Self> {
        P::parse(input)
    }
}
