use parcom_core::{primitive::BytesDelta, Parser, ParserOnce, SegmentStream, Sequence};
use parcom_util::{done, error::Miss, fail};

pub fn the_char(c: char) -> TheChar {
    TheChar { c }
}
pub struct TheChar {
    c: char,
}

impl<S: Sequence<Segment = str, Length = BytesDelta>> ParserOnce<S> for TheChar {
    type Output = ();
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> parcom_core::ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<S: Sequence<Segment = str, Length = BytesDelta>> Parser<S> for TheChar {
    async fn parse(&self, mut input: S) -> parcom_core::ParserResult<S, Self> {
        let mut segments = input.segments();

        while let Some(segment) = segments.next(BytesDelta::from_char(self.c)).await {
            let Some(c) = segment.chars().next() else {
                continue;
            };

            if c == self.c {
                drop(segments);
                return done((), input.advance(BytesDelta::from_char(self.c)).await);
            }

            break;
        }

        drop(segments);
        fail((), input)
    }
}

pub fn the_item<T: PartialEq>(item: T) -> TheItem<T> {
    TheItem { item }
}

pub struct TheItem<T: PartialEq> {
    item: T,
}

impl<T: 'static + PartialEq, S: Sequence<Segment = [T], Length = usize>> ParserOnce<S>
    for TheItem<T>
{
    type Output = ();
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> parcom_core::ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<T: 'static + PartialEq, S: Sequence<Segment = [T], Length = usize>> Parser<S> for TheItem<T> {
    async fn parse(&self, mut input: S) -> parcom_core::ParserResult<S, Self> {
        let mut segments = input.segments();

        while let Some(segment) = segments.next(1).await {
            let Some(item) = segment.iter().next() else {
                continue;
            };

            if item == &self.item {
                drop(segments);
                return done((), input.advance(1).await);
            }

            break;
        }

        drop(segments);
        fail((), input)
    }
}
