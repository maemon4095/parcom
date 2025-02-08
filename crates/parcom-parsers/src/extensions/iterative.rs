use parcom_core::{IterativeParser, RewindStream};

use crate::iterative::{scan::Scan, Collect, Fold, MapEach, MapWhile, Take};

pub trait IterativeParserExtension<S>: IterativeParser<S> {
    fn collect<C: Extend<Self::Output> + Default>(self) -> Collect<S, Self, C>
    where
        S: RewindStream,
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

impl<S, P: IterativeParser<S>> IterativeParserExtension<S> for P {}
