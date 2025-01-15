mod control;
mod controller;
mod controllers;

use parcom_core::{ParseResult, Parser, RewindStream};
use std::marker::PhantomData;

pub use control::{RepeatContol, RepeatFailControl};
pub use controller::{RepeatContollerResult, RepeatController};

pub struct Repeat<S: RewindStream, P: Parser<S>, C: RepeatController<S, P>> {
    parser: P,
    controller: C,
    marker: PhantomData<S>,
}

impl<S: RewindStream, P: Parser<S>, C: RepeatController<S, P>> Repeat<S, P, C> {
    pub fn new(parser: P, controller: C) -> Self {
        Self {
            parser,
            controller,
            marker: PhantomData,
        }
    }
}

impl<S: RewindStream, P: Parser<S>, C: RepeatController<S, P>> Parser<S> for Repeat<S, P, C> {
    type Output = C::Output;
    type Error = C::Error;

    async fn parse(&self, input: S) -> ParseResult<S, Self::Output, Self::Error> {
        let control = RepeatContol::new(&self.parser, input);
        self.controller.control(control).await.result
    }
}
