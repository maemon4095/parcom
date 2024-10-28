use std::ops::Deref;

use futures::StreamExt as _;
use parcom_core::{Never, ParcomStream, ParseResult::*, Parser, ParserResult};

pub fn atom<T>(items: &[T]) -> Atom<'_, T>
where
    T: PartialEq,
{
    Atom { items }
}

pub fn single<'a, T>(item: &'a T) -> Single<'a, T>
where
    T: PartialEq,
{
    Single { item }
}

pub struct Atom<'a, T>
where
    T: PartialEq,
{
    items: &'a [T],
}

impl<'a, T, S> Parser<S> for Atom<'a, T>
where
    T: PartialEq,
    S: ParcomStream<Segment = [T]>,
{
    type Output = &'a [T];
    type Error = ();
    type Fault = Never;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut remain = self.items;
        let mut segments = input.segments();

        while let Some(segment) = segments.next().await {
            let segment = segment.deref();

            if !segment.starts_with(&remain) {
                break;
            }

            if segment.len() >= remain.len() {
                return Done(self.items, input.advance(self.items.len()).await);
            }

            remain = &remain[segment.len()..];
        }

        return Fail((), input.into());
    }
}

pub struct Single<'a, T>
where
    T: PartialEq,
{
    item: &'a T,
}

impl<'a, T, S> Parser<S> for Single<'a, T>
where
    T: PartialEq,
    S: ParcomStream<Segment = [T]>,
{
    type Output = &'a T;
    type Error = ();
    type Fault = Never;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        loop {
            let Some(segment) = segments.next().await else {
                break;
            };

            if let Some(c) = segment.first() {
                if c == self.item {
                    return Done(self.item, input.advance(1).await);
                }

                break;
            }
        }

        Fail((), input.into())
    }
}
