use parcom_core::{IterativeParser, ParseError, RewindSequence, Sequence};

use crate::iterative::{Collect, Fold, MapEach, MapWhile, Scan, Take, TryMapEach};

pub trait IterativeParserExtension<S: Sequence>: IterativeParser<S> {
    fn collect<C: Extend<Self::Output> + Default>(self) -> Collect<S, Self, C>
    where
        S: RewindSequence,
        Self: Sized,
    {
        Collect::new(self)
    }

    fn fold<A, F>(self, init: A, f: F) -> Fold<S, Self, A, F>
    where
        Self: Sized,
        F: Fn(A, Self::Output) -> A,
    {
        Fold::new(self, init, f)
    }

    fn scan<St, O, F>(self, initial_state: St, f: F) -> Scan<S, Self, St, F>
    where
        Self: Sized,
        F: Fn(&mut St, Self::Output) -> O,
    {
        Scan::new(self, initial_state, f)
    }

    fn map_each<T, F: Fn(Self::Output) -> T>(self, map: F) -> MapEach<S, Self, F>
    where
        Self: Sized,
    {
        MapEach::new(self, map)
    }

    fn try_map_each<O, E, F>(self, f: F) -> TryMapEach<S, Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Result<O, E>,
        E: From<Self::Error> + ParseError,
    {
        TryMapEach::new(self, f)
    }

    fn map_while<T, F: Fn(Self::Output) -> Option<T>>(self, map: F) -> MapWhile<S, Self, F>
    where
        Self: Sized,
    {
        MapWhile::new(self, map)
    }

    fn take(self, count: usize) -> Take<S, Self>
    where
        Self: Sized,
    {
        Take::new(self, count)
    }
}

impl<S: Sequence, P: IterativeParser<S>> IterativeParserExtension<S> for P {}
