use crate::internal::just_on_boundary;
use parcom_core::{
    Never,
    ParseResult::{self, *},
    Parser, ParserResult, RewindStream,
};
use std::{marker::PhantomData, ops::RangeBounds};

pub struct Repeat<T: RewindStream, P: Parser<T>, R: RepeatBounds<T, P>> {
    pub(super) range: R,
    pub(super) parser: P,
    pub(super) marker: PhantomData<T>,
}

impl<S: RewindStream, P: Parser<S>, R: RepeatBounds<S, P>> Parser<S> for Repeat<S, P, R> {
    type Output = (Vec<P::Output>, Option<P::Error>);
    type Error = R::Error;
    type Fault = P::Fault;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        R::parse(self, input)
    }
}

fn default_parse<S, P: Parser<S>, R: RepeatBounds<S, P>>(
    me: &Repeat<S, P, R>,
    input: S,
) -> ParseResult<S, (Vec<P::Output>, Option<P::Error>), P::Error, P::Fault>
where
    S: RewindStream,
{
    let mut vec = Vec::new();
    let upper_bound = me.range.end_bound();

    let mut rest = input;
    let (last_error, rest) = loop {
        if just_on_boundary(vec.len(), upper_bound) {
            break (None, rest);
        }

        let (e, r) = {
            let anchor = rest.anchor();
            match me.parser.parse(rest) {
                Done(v, r) => (v, r),
                Fail(e, r) => break (Some(e), r.rewind(anchor)),
                Fatal(e, r) => return Fatal(e, r),
            }
        };

        vec.push(e);
        rest = r;
    };

    if me.range.contains(&vec.len()) {
        Done((vec, last_error), rest)
    } else {
        Fail(last_error.unwrap(), rest.into())
    }
}

pub trait RepeatBounds<S: RewindStream, P: Parser<S>>: Sized + RangeBounds<usize> {
    type Error;

    fn parse(me: &Repeat<S, P, Self>, input: S) -> ParserResult<S, Repeat<S, P, Self>>;
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::RangeFull {
    type Error = Never;

    fn parse(me: &Repeat<S, P, Self>, input: S) -> ParserResult<S, Repeat<S, P, Self>>
    where
        S: RewindStream,
    {
        let mut vec = Vec::new();
        let upper_bound = me.range.end_bound();

        let mut rest = input;
        let (last_error, rest) = loop {
            if just_on_boundary(vec.len(), upper_bound) {
                break (None, rest);
            }

            let (e, r) = {
                let anchor = rest.anchor();
                match me.parser.parse(rest) {
                    Done(v, r) => (v, r),
                    Fail(e, r) => break (Some(e), r.rewind(anchor)),
                    Fatal(e, r) => return Fatal(e, r),
                }
            };

            vec.push(e);
            rest = r;
        };

        Done((vec, last_error), rest)
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::Range<usize> {
    type Error = P::Error;

    fn parse(me: &Repeat<S, P, Self>, input: S) -> ParserResult<S, Repeat<S, P, Self>>
    where
        S: RewindStream,
    {
        default_parse(me, input)
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::RangeFrom<usize> {
    type Error = P::Error;

    fn parse(me: &Repeat<S, P, Self>, input: S) -> ParserResult<S, Repeat<S, P, Self>>
    where
        S: RewindStream,
    {
        default_parse(me, input)
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::RangeTo<usize> {
    type Error = P::Error;

    fn parse(me: &Repeat<S, P, Self>, input: S) -> ParserResult<S, Repeat<S, P, Self>>
    where
        S: RewindStream,
    {
        default_parse(me, input)
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatBounds<S, P> for std::ops::RangeToInclusive<usize> {
    type Error = P::Error;

    fn parse(me: &Repeat<S, P, Self>, input: S) -> ParserResult<S, Repeat<S, P, Self>>
    where
        S: RewindStream,
    {
        default_parse(me, input)
    }
}
