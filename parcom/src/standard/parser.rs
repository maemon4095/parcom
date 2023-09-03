mod as_ref;
mod join;
mod maps;
mod optional;
mod or;
mod repeat;
mod repeat_n;
use self::{
    as_ref::AsRef,
    join::Join,
    maps::{Map, MapErr},
    optional::Optional,
    or::Or,
    repeat::Repeat,
    repeat_n::RepeatN,
};
use std::{marker::PhantomData, ops::RangeBounds};

use crate::{internal::Sealed, Parser, RewindStream};

pub trait ParserExtension<T>: Parser<T> + Sealed {
    fn repeat<R: RangeBounds<usize>>(self, range: R) -> Repeat<T, Self, R>
    where
        Self: Sized,
        T: RewindStream,
    {
        Repeat {
            range,
            parser: self,
            marker: PhantomData,
        }
    }

    fn repeat_n<const N: usize>(self) -> RepeatN<T, Self, N>
    where
        Self: Sized,
        T: RewindStream,
    {
        RepeatN {
            parser: self,
            marker: PhantomData,
        }
    }

    fn optional(self) -> Optional<T, Self>
    where
        Self: Sized,
        T: RewindStream,
    {
        Optional {
            parser: self,
            marker: PhantomData,
        }
    }

    fn or<P: Parser<T>>(self, other: P) -> Or<T, Self, P>
    where
        Self: Sized,
        T: RewindStream,
    {
        Or {
            parser0: self,
            parser1: other,
            marker: PhantomData,
        }
    }

    fn join<P: Parser<T>>(self, other: P) -> Join<T, Self, P>
    where
        Self: Sized,
        T: RewindStream,
    {
        Join {
            parser0: self,
            parser1: other,
            marker: PhantomData,
        }
    }

    fn map<U, F: Fn(Self::Output) -> U>(self, mapping: F) -> Map<T, Self, U, F>
    where
        Self: Sized,
        T: RewindStream,
    {
        Map {
            parser: self,
            mapping,
            marker: PhantomData,
        }
    }

    fn map_err<U, F: Fn(Self::Error) -> U>(self, mapping: F) -> MapErr<T, Self, U, F>
    where
        Self: Sized,
        T: RewindStream,
    {
        MapErr {
            parser: self,
            mapping,
            marker: PhantomData,
        }
    }

    fn as_ref(&self) -> AsRef<'_, T, Self>
    where
        Self: Sized,
    {
        AsRef {
            parser: self,
            marker: PhantomData,
        }
    }
}

impl<T, P: Parser<T> + Sealed> ParserExtension<T> for P {}
