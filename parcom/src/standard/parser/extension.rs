mod as_ref;
mod discard;
mod fold;

mod join;
mod maps;
mod never_fault;
mod optional;
mod or;
mod repeat;
mod repeat_n;
mod unify;
pub use self::{
    as_ref::AsRef,
    discard::Discard,
    fold::Fold,
    join::Join,
    maps::{Map, MapErr},
    never_fault::NeverFault,
    optional::Optional,
    or::Or,
    repeat::Repeat,
    repeat_n::RepeatN,
    unify::{Unify, UnifyErr, UnifyFault},
};

use std::{marker::PhantomData, ops::RangeBounds};

use crate::{internal::Sealed, Parser, RewindStream};

use crate::standard::Either;

pub trait ParserExtension<S>: Parser<S> + Sealed {
    fn repeat<R: RangeBounds<usize>>(self, range: R) -> Repeat<S, Self, R>
    where
        Self: Sized,
        S: RewindStream,
    {
        Repeat {
            range,
            parser: self,
            marker: PhantomData,
        }
    }

    fn repeat_n<const N: usize>(self) -> RepeatN<S, Self, N>
    where
        Self: Sized,
        S: RewindStream,
    {
        RepeatN {
            parser: self,
            marker: PhantomData,
        }
    }

    fn optional(self) -> Optional<S, Self>
    where
        Self: Sized,
        S: RewindStream,
    {
        Optional {
            parser: self,
            marker: PhantomData,
        }
    }

    fn or<P: Parser<S>>(self, other: P) -> Or<S, Self, P>
    where
        Self: Sized,
        S: RewindStream,
    {
        Or {
            parser0: self,
            parser1: other,
            marker: PhantomData,
        }
    }

    fn join<P: Parser<S>>(self, other: P) -> Join<S, Self, P>
    where
        Self: Sized,
        S: RewindStream,
    {
        Join {
            parser0: self,
            parser1: other,
            marker: PhantomData,
        }
    }

    fn map<U, F: Fn(Self::Output) -> U>(self, mapping: F) -> Map<S, Self, U, F>
    where
        Self: Sized,
        S: RewindStream,
    {
        Map {
            parser: self,
            mapping,
            marker: PhantomData,
        }
    }

    fn map_err<U, F: Fn(Self::Error) -> U>(self, mapping: F) -> MapErr<S, Self, U, F>
    where
        Self: Sized,
        S: RewindStream,
    {
        MapErr {
            parser: self,
            mapping,
            marker: PhantomData,
        }
    }

    fn as_ref(&self) -> AsRef<'_, S, Self> {
        AsRef {
            parser: self,
            marker: PhantomData,
        }
    }

    fn unify<T>(self) -> Unify<S, T, Self>
    where
        Self: Sized + Parser<S, Output = Either<T, T>>,
    {
        Unify {
            parser: self,
            marker: PhantomData,
        }
    }

    fn unify_err<T>(self) -> UnifyErr<S, T, Self>
    where
        Self: Sized + Parser<S, Error = Either<T, T>>,
    {
        UnifyErr {
            parser: self,
            marker: PhantomData,
        }
    }

    fn unify_fault<T>(self) -> UnifyFault<S, T, Self>
    where
        Self: Sized + Parser<S, Fault = Either<T, T>>,
    {
        UnifyFault {
            parser: self,
            marker: PhantomData,
        }
    }

    fn fold<A, FInit, FBody>(self, init: FInit) -> Fold<S, Self, A, FInit, FBody>
    where
        Self: Sized,
        FInit: Fn() -> (A, FBody),
        FBody: FnMut(A, Self::Output) -> A,
    {
        Fold {
            parser: self,
            init,
            marker: PhantomData,
        }
    }

    fn discard(self) -> Discard<S, Self>
    where
        Self: Sized,
    {
        Discard {
            parser: self,
            marker: PhantomData,
        }
    }

    fn never_fault(self) -> NeverFault<S, Self>
    where
        Self: Sized,
    {
        NeverFault {
            parser: self,
            marker: PhantomData,
        }
    }
}

impl<S, P: Parser<S> + Sealed> ParserExtension<S> for P {}
