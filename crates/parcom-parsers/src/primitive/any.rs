use std::marker::PhantomData;

use parcom_core::{
    primitive::BytesDelta, Parser, ParserOnce, ParserResult, SegmentIterator, Stream,
};
use parcom_util::{done, error::Miss, fail, ResultExt};

pub fn any_char<S: Stream<Segment = str>>() -> AnyChar<S> {
    AnyChar::new()
}

pub struct AnyChar<S: Stream<Segment = str>>(PhantomData<S>);

impl<S: Stream<Segment = str>> AnyChar<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: Stream<Segment = str>> ParserOnce<S> for AnyChar<S> {
    type Output = char;
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<S: Stream<Segment = str>> Parser<S> for AnyChar<S> {
    async fn parse(&self, mut input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        while let Some(segment) = segments.next(BytesDelta::ZERO).await {
            let segment = segment.stream_err()?;

            let Some(c) = segment.chars().next() else {
                continue;
            };
            return done(
                c,
                input.advance(BytesDelta::from_char(c)).await.stream_err()?,
            );
        }

        drop(segments);
        fail(().into(), input)
    }
}

pub fn any_item<T: Clone, S: Stream<Segment = [T]>>() -> AnyItem<T, S> {
    AnyItem::new()
}

pub struct AnyItem<T: Clone, S: Stream<Segment = [T]>>(PhantomData<(T, S)>);

impl<T: Clone, S: Stream<Segment = [T]>> AnyItem<T, S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Clone, S: Stream<Segment = [T]>> ParserOnce<S> for AnyItem<T, S> {
    type Output = T;
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<T: Clone, S: Stream<Segment = [T]>> Parser<S> for AnyItem<T, S> {
    async fn parse(&self, mut input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        while let Some(segment) = segments.next(1).await {
            let segment = segment.stream_err()?;
            if segment.is_empty() {
                continue;
            }

            return done(segment[0].clone(), input.advance(1).await.stream_err()?);
        }

        fail(().into(), input)
    }
}
