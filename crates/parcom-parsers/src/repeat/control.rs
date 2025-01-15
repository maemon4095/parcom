use super::{RepeatContollerResult, RepeatController};
use parcom_core::{ParseResult::*, Parser, RewindStream, UnknownLocation};
use std::marker::PhantomData;

pub struct RepeatContol<'a, S, P, C>
where
    S: RewindStream,
    P: Parser<S>,
    C: RepeatController<S, P>,
{
    parser: &'a P,
    stream: S,
    marker: PhantomData<C>,
}

impl<'a, S, P, C> RepeatContol<'a, S, P, C>
where
    S: RewindStream,
    P: Parser<S>,
    C: RepeatController<S, P>,
{
    pub(super) fn new(parser: &'a P, stream: S) -> Self {
        Self {
            parser,
            stream,
            marker: PhantomData,
        }
    }

    pub async fn next(
        mut self,
    ) -> Result<(P::Output, Self), (P::Error, RepeatFailControl<S, P, C>)> {
        match self.parser.parse(self.stream).await {
            Done(v, r) => {
                self.stream = r;
                Ok((v, self))
            }
            Fail(e, r) => {
                let c = RepeatFailControl {
                    stream: r,
                    marker: PhantomData,
                };
                Err((e, c))
            }
        }
    }

    pub fn anchor(&self) -> S::Anchor {
        self.stream.anchor()
    }

    pub fn done(self, output: C::Output) -> RepeatContollerResult<S, P, C> {
        let result = Done(output, self.stream);
        RepeatContollerResult { result }
    }

    pub async fn done_halfway(
        self,
        output: C::Output,
        anchor: S::Anchor,
    ) -> RepeatContollerResult<S, P, C> {
        let r = self.stream.rewind(anchor).await;
        let result = Done(output, r);
        RepeatContollerResult { result }
    }

    pub fn fail(self, error: C::Error) -> RepeatContollerResult<S, P, C> {
        let result = Fail(error, self.stream.into());
        RepeatContollerResult { result }
    }
}

pub struct RepeatFailControl<S, P, C: RepeatController<S, P>>
where
    S: RewindStream,
    P: Parser<S>,
{
    stream: UnknownLocation<S>,
    marker: PhantomData<(P, C)>,
}

impl<'a, S: RewindStream, P: Parser<S>, C: RepeatController<S, P>> RepeatFailControl<S, P, C> {
    pub async fn done(
        self,
        output: C::Output,
        anchor: S::Anchor,
    ) -> RepeatContollerResult<S, P, C> {
        let r = self.stream.rewind(anchor).await;
        let result = Done(output, r);
        RepeatContollerResult { result }
    }

    pub fn fail(self, error: C::Error) -> RepeatContollerResult<S, P, C> {
        let result = Fail(error, self.stream);
        RepeatContollerResult { result }
    }
}
