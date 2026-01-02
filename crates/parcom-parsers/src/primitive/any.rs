use parcom_core::{
    primitive::BytesDelta, Parser, ParserOnce, ParserResult, SegmentStream, Sequence,
};
use parcom_util::{done, error::Miss, fail};
use std::marker::PhantomData;

pub fn any_char<S: Sequence<Segment = str>>() -> AnyChar<S> {
    AnyChar::new()
}

pub struct AnyChar<S: Sequence<Segment = str>>(PhantomData<S>);

impl<S: Sequence<Segment = str>> AnyChar<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: Sequence<Segment = str, Length = BytesDelta>> ParserOnce<S> for AnyChar<S> {
    type Output = char;
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<S: Sequence<Segment = str, Length = BytesDelta>> Parser<S> for AnyChar<S> {
    async fn parse(&self, mut input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();
        while let Some(segment) = segments.next(BytesDelta::from_bytes(0)).await {
            let Some(c) = segment.chars().next() else {
                continue;
            };

            drop(segments);
            return done(c, input.advance(BytesDelta::from_char(c)).await);
        }

        drop(segments);
        fail((), input)
    }
}

pub fn any_item<T: 'static + Clone, S: Sequence<Segment = [T]>>() -> AnyItem<T, S> {
    AnyItem::new()
}

pub struct AnyItem<T: 'static + Clone, S: Sequence<Segment = [T]>>(PhantomData<(T, S)>);

impl<T: Clone, S: Sequence<Segment = [T]>> AnyItem<T, S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: 'static + Clone, S: Sequence<Segment = [T], Length = usize>> ParserOnce<S>
    for AnyItem<T, S>
{
    type Output = T;
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<T: 'static + Clone, S: Sequence<Segment = [T], Length = usize>> Parser<S> for AnyItem<T, S> {
    async fn parse(&self, mut input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        while let Some(segment) = segments.next(0).await {
            if segment.is_empty() {
                continue;
            }

            let item = segment[0].clone();
            drop(segments);
            return done(item, input.advance(1).await);
        }

        drop(segments);
        fail((), input)
    }
}
