use crate::{util::Boxed, AndThen, Join, Map, MapErr, Optional, Or, Ref, Repeat, Unify, UnifyErr};
use parcom_core::{ParseError, Parser, ParserOnce, RewindStream, Stream};
use parcom_util::Either;

pub trait ParserExtension<S: Stream>: ParserOnce<S> {
    fn optional(self) -> Optional<S, Self>
    where
        Self: Sized,
        S: RewindStream,
    {
        Optional::new(self)
    }

    fn or<P: ParserOnce<S>>(self, other: P) -> Or<S, Self, P>
    where
        Self: Sized,
        S: RewindStream,
    {
        Or::new(self, other)
    }

    fn join<P: ParserOnce<S>>(self, other: P) -> Join<S, Self, P>
    where
        Self: Sized,
        S: RewindStream,
    {
        Join::new(self, other)
    }

    fn map<U, F: Fn(Self::Output) -> U>(self, mapping: F) -> Map<S, Self, U, F>
    where
        Self: Sized,
    {
        Map::new(self, mapping)
    }

    fn map_err<U: ParseError, F: Fn(Self::Error) -> U>(self, mapping: F) -> MapErr<S, Self, U, F>
    where
        Self: Sized,
    {
        MapErr::new(self, mapping)
    }

    fn as_ref(&self) -> Ref<'_, S, Self>
    where
        Self: Parser<S>,
    {
        Ref::new(self)
    }

    fn unify<T0, T1, T>(self) -> Unify<S, T0, T1, T, Self>
    where
        Self: Sized + Parser<S, Output = Either<T0, T1>>,
        T0: Into<T>,
        T1: Into<T>,
    {
        Unify::new(self)
    }

    fn unify_err<T0, T1, T>(self) -> UnifyErr<S, T0, T1, T, Self>
    where
        Self: Sized + Parser<S, Error = Either<T0, T1>>,
        T0: Into<T>,
        T1: Into<T>,
    {
        UnifyErr::new(self)
    }

    fn repeat(self) -> Repeat<S, Self>
    where
        S: RewindStream,
        Self: Sized + Parser<S>,
    {
        Repeat::new(self)
    }

    fn and_then<O, E, F>(self, map: F) -> AndThen<S, Self, O, E, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> Result<O, E>,
        E: ParseError + From<Self::Error>,
    {
        AndThen::new(self, map)
    }

    fn boxed(self) -> Boxed<S, Self>
    where
        Self: Sized,
    {
        Boxed::new(self)
    }
}

impl<S: Stream, P: Parser<S>> ParserExtension<S> for P {}
