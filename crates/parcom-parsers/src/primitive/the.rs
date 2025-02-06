use parcom_base::error::Miss;
use parcom_core::{ParseResult, Parser, ParserOnce, SegmentIterator, Stream};

pub fn the_char(c: char) -> TheChar {
    TheChar { c }
}
pub struct TheChar {
    c: char,
}

impl<S: Stream<Segment = str>> ParserOnce<S> for TheChar {
    type Output = ();
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> parcom_core::ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<S: Stream<Segment = str>> Parser<S> for TheChar {
    async fn parse(&self, input: S) -> parcom_core::ParserResult<S, Self> {
        let mut segments = input.segments();

        while let Some(segment) = segments.next(self.c.len_utf8().into()).await {
            let Some(c) = segment.chars().next() else {
                continue;
            };

            if c == self.c {
                return ParseResult::Done((), input.advance(self.c.len_utf8().into()).await);
            }

            break;
        }

        ParseResult::Fail(().into(), input.into())
    }
}

pub fn the_item<T: PartialEq>(item: T) -> TheItem<T> {
    TheItem { item }
}

pub struct TheItem<T: PartialEq> {
    item: T,
}

impl<T: PartialEq, S: Stream<Segment = [T]>> ParserOnce<S> for TheItem<T> {
    type Output = ();
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> parcom_core::ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<T: PartialEq, S: Stream<Segment = [T]>> Parser<S> for TheItem<T> {
    async fn parse(&self, input: S) -> parcom_core::ParserResult<S, Self> {
        let mut segments = input.segments();

        while let Some(segment) = segments.next(1).await {
            let Some(item) = segment.iter().next() else {
                continue;
            };

            if item == &self.item {
                return ParseResult::Done((), input.advance(1).await);
            }

            break;
        }

        ParseResult::Fail(().into(), input.into())
    }
}
