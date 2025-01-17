use parcom_base::error::Miss;
use parcom_core::{ParseResult::*, Parser, ParserResult, SegmentIterator, Stream};
use std::{marker::PhantomData, ops::Deref};

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

pub fn any_item<T: Clone>() -> AnyItem<T> {
    AnyItem {
        marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct Atom<'a, T>
where
    T: PartialEq,
{
    items: &'a [T],
}

impl<'a, T, S> Parser<S> for Atom<'a, T>
where
    T: PartialEq,
    S: Stream<Segment = [T]>,
{
    type Output = &'a [T];
    type Error = Miss<()>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut remain = self.items;
        let mut segments = input.segments();

        while let Some(segment) = segments.next(remain.len()).await {
            let segment = segment.deref();

            if !segment.starts_with(&remain) {
                break;
            }

            if segment.len() >= remain.len() {
                return Done(self.items, input.advance(self.items.len()).await);
            }

            remain = &remain[segment.len()..];
        }

        return Fail(().into(), input.into());
    }
}

#[derive(Debug)]
pub struct Single<'a, T>
where
    T: PartialEq,
{
    item: &'a T,
}

impl<'a, T, S> Parser<S> for Single<'a, T>
where
    T: PartialEq,
    S: Stream<Segment = [T]>,
{
    type Output = &'a T;
    type Error = Miss<()>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        loop {
            let Some(segment) = segments.next(1).await else {
                break;
            };

            if let Some(c) = segment.first() {
                if c == self.item {
                    return Done(self.item, input.advance(1).await);
                }

                break;
            }
        }

        Fail(().into(), input.into())
    }
}

#[derive(Debug)]
pub struct AnyItem<T: Clone> {
    marker: PhantomData<T>,
}

impl<T: Clone, S: Stream<Segment = [T]>> Parser<S> for AnyItem<T> {
    type Output = T;
    type Error = Miss<()>;

    async fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut segments = input.segments();

        loop {
            let Some(segment) = segments.next(1).await else {
                break Fail(().into(), input.into());
            };

            if let Some(c) = segment.first() {
                let rest = input.advance(1).await;
                break Done(c.clone(), rest);
            }
        }
    }
}
