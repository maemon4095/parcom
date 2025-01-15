mod as_ref;
mod discard;
mod fold;
mod join;
mod maps;
mod optional;
mod or;
mod repeat_n;
mod repeat_range;
mod then;
mod unify;

pub use self::{
    as_ref::AsRef,
    discard::Discard,
    fold::Fold,
    join::Join,
    maps::{Map, MapErr},
    optional::Optional,
    or::Or,
    repeat_n::RepeatN,
    repeat_range::RepeatRange,
    unify::{Unify, UnifyErr},
};
use parcom_base::Either;
use parcom_core::{ParseError, ParseResult, Parser, ParserResult, RewindStream};
use repeat_range::RepeatBounds;
use then::Then;

pub trait ParserExtension<S>: Parser<S> {
    fn repeat_range<R: RepeatBounds<S, Self>>(self, range: R) -> RepeatRange<S, Self, R>
    where
        Self: Sized,
        S: RewindStream,
    {
        RepeatRange::new(self, range)
    }

    fn repeat_n<const N: usize>(self) -> RepeatN<S, Self, N>
    where
        Self: Sized,
        S: RewindStream,
    {
        RepeatN::new(self)
    }

    fn optional(self) -> Optional<S, Self>
    where
        Self: Sized,
        S: RewindStream,
    {
        Optional::new(self)
    }

    fn or<P: Parser<S>>(self, other: P) -> Or<S, Self, P>
    where
        Self: Sized,
        S: RewindStream,
    {
        Or::new(self, other)
    }

    fn join<P: Parser<S>>(self, other: P) -> Join<S, Self, P>
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

    fn as_ref(&self) -> AsRef<'_, S, Self> {
        AsRef::new(self)
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

    fn fold<A, FInit, FBody>(self, init: FInit) -> Fold<S, Self, A, FInit, FBody>
    where
        S: RewindStream,
        Self: Sized,
        FInit: Fn() -> (A, FBody),
        FBody: FnMut(A, Self::Output) -> A,
    {
        Fold::new(self, init)
    }

    fn discard(self) -> Discard<S, Self>
    where
        Self: Sized,
    {
        Discard::new(self)
    }

    fn then<O, E: ParseError, Fun>(self, f: Fun) -> Then<S, Self, O, E, Fun>
    where
        Self: Sized,
        Fun: Fn(ParserResult<S, Self>) -> ParseResult<S, O, E>,
    {
        Then::new(self, f)
    }
}

impl<S, P: Parser<S>> ParserExtension<S> for P {}
