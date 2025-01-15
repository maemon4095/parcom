use std::future::Future;

use parcom_core::{ParseError, ParseResult, Parser, RewindStream};

use super::RepeatContol;

pub struct RepeatContollerResult<S: RewindStream, P: Parser<S>, C: RepeatController<S, P>> {
    pub(super) result: ParseResult<S, C::Output, C::Error>,
}

pub trait RepeatController<S: RewindStream, P: Parser<S>>: Sized {
    type Output;
    type Error: ParseError;

    fn control(
        &self,
        control: RepeatContol<'_, S, P, Self>,
    ) -> impl Future<Output = RepeatContollerResult<S, P, Self>>;
}
