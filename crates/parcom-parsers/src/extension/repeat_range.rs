use crate::internal::just_on_boundary;
use parcom_core::{
    ParseError,
    ParseResult::{self, *},
    Parser, ParserResult, RewindStream,
};
use std::{marker::PhantomData, ops::RangeBounds};

pub struct RepeatRange<T: RewindStream, P: Parser<T>, R: RepeatBounds<T, P>> {
    parser: P,
    range: R,
    marker: PhantomData<T>,
}

impl<T: RewindStream, P: Parser<T>, R: RepeatBounds<T, P>> RepeatRange<T, P, R> {
    pub fn new(parser: P, range: R) -> Self {
        Self {
            range,
            parser,
            marker: PhantomData,
        }
    }
}

impl<S: RewindStream, P: Parser<S>, R: RepeatBounds<S, P>> Parser<S> for RepeatRange<S, P, R> {
    type Output = R::Output;
    type Error = R::Error;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        R::parse(self, input).await
    }
}

async fn default_parse<S, P: Parser<S>, R: RepeatBounds<S, P>>(
    me: &RepeatRange<S, P, R>,
    input: S,
) -> ParseResult<S, (Vec<P::Output>, Option<P::Error>), P::Error>
where
    S: RewindStream,
{
    let start_bound = me.range.start_bound();
    let capacity = match start_bound {
        std::ops::Bound::Included(n) => *n,
        std::ops::Bound::Excluded(n) => n.checked_sub(1).unwrap_or(0),
        std::ops::Bound::Unbounded => 0,
    };
    let mut vec = Vec::with_capacity(capacity);
    let upper_bound = me.range.end_bound();

    let mut rest = input;
    let (last_error, rest) = loop {
        if just_on_boundary(vec.len(), upper_bound) {
            return Done((vec, None), rest);
        }

        let (e, r) = {
            let anchor = rest.anchor();
            match me.parser.parse(rest).await {
                Done(v, r) => (v, r),
                Fail(e, r) if !e.should_terminate() => break (e, r.rewind(anchor).await),
                Fail(e, r) => return Fail(e, r),
            }
        };

        vec.push(e);
        rest = r;
    };

    if me.range.contains(&vec.len()) {
        Done((vec, Some(last_error)), rest)
    } else {
        Fail(last_error, rest.into())
    }
}

pub trait RepeatBounds<S: RewindStream, P: Parser<S>>: Sized + RangeBounds<usize> {
    type Output;
    type Error: ParseError;

    fn parse(
        me: &RepeatRange<S, P, Self>,
        input: S,
    ) -> impl std::future::Future<Output = ParserResult<S, RepeatRange<S, P, Self>>>;
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::RangeFull {
    type Output = (Vec<P::Output>, P::Error);
    type Error = P::Error;

    async fn parse(
        me: &RepeatRange<S, P, Self>,
        input: S,
    ) -> ParserResult<S, RepeatRange<S, P, Self>>
    where
        S: RewindStream,
    {
        let mut vec = Vec::new();
        let mut rest = input;
        let (last_error, rest) = loop {
            let (e, r) = {
                let anchor = rest.anchor();
                match me.parser.parse(rest).await {
                    Done(v, r) => (v, r),
                    Fail(e, r) if !e.should_terminate() => break (e, r.rewind(anchor).await),
                    Fail(e, r) => return Fail(e, r),
                }
            };

            vec.push(e);
            rest = r;
        };

        Done((vec, last_error), rest)
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::RangeFrom<usize> {
    type Output = (Vec<P::Output>, P::Error);
    type Error = P::Error;

    async fn parse(
        me: &RepeatRange<S, P, Self>,
        input: S,
    ) -> ParserResult<S, RepeatRange<S, P, Self>>
    where
        S: RewindStream,
    {
        let mut vec = Vec::new();
        let mut rest = input;
        let (last_error, rest) = loop {
            let (e, r) = {
                let anchor = rest.anchor();
                match me.parser.parse(rest).await {
                    Done(v, r) => (v, r),
                    Fail(e, r) if !e.should_terminate() => break (e, r.rewind(anchor).await),
                    Fail(e, r) => return Fail(e, r),
                }
            };

            vec.push(e);
            rest = r;
        };

        if me.range.contains(&vec.len()) {
            Done((vec, last_error), rest)
        } else {
            Fail(last_error, rest.into())
        }
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::Range<usize> {
    type Output = (Vec<P::Output>, Option<P::Error>);
    type Error = P::Error;

    async fn parse(
        me: &RepeatRange<S, P, Self>,
        input: S,
    ) -> ParserResult<S, RepeatRange<S, P, Self>>
    where
        S: RewindStream,
    {
        default_parse(me, input).await
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::RangeTo<usize> {
    type Output = (Vec<P::Output>, Option<P::Error>);
    type Error = P::Error;

    async fn parse(
        me: &RepeatRange<S, P, Self>,
        input: S,
    ) -> ParserResult<S, RepeatRange<S, P, Self>>
    where
        S: RewindStream,
    {
        default_parse(me, input).await
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::RangeToInclusive<usize> {
    type Output = (Vec<P::Output>, Option<P::Error>);
    type Error = P::Error;

    async fn parse(
        me: &RepeatRange<S, P, Self>,
        input: S,
    ) -> ParserResult<S, RepeatRange<S, P, Self>>
    where
        S: RewindStream,
    {
        default_parse(me, input).await
    }
}
