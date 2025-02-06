use parcom_core::{IterativeParser, RewindStream};

use crate::iterative::{Collect, MapEach, Take};

pub trait IterativeParserExtension<S>: IterativeParser<S> {
    fn collect<C: Extend<Self::Output> + Default>(self) -> Collect<S, Self, C>
    where
        S: RewindStream,
        Self: Sized,
    {
        Collect::new(self)
    }

    fn map_each<T, F: Fn(Self::Output) -> T>(self, map: F) -> MapEach<S, Self, T, F>
    where
        Self: Sized,
    {
        MapEach::new(self, map)
    }

    fn take(self, count: usize) -> Take<S, Self>
    where
        Self: Sized,
    {
        Take::new(self, count)
    }
}

impl<S, P: IterativeParser<S>> IterativeParserExtension<S> for P {}
