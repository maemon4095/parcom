mod join;
mod maps;
mod optional;
mod or;
mod repeat;
mod repeat_n;
use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use crate::{internal::Sealed, ParseStream};

use self::{
    join::Join,
    maps::{Map, MapErr},
    optional::Optional,
    or::Or,
    repeat::Repeat,
    repeat_n::RepeatN,
};

use super::Parser;

pub trait StandardExtension<T: ParseStream>: Parser<T> + Sealed {
    fn repeat<R: RangeBounds<usize>>(self, range: R) -> Repeat<T, Self, R>
    where
        Self: Sized,
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
    {
        RepeatN {
            parser: self,
            marker: PhantomData,
        }
    }

    fn optional(self) -> Optional<T, Self>
    where
        Self: Sized,
    {
        Optional {
            parser: self,
            marker: PhantomData,
        }
    }

    fn or<P: Parser<T>>(self, other: P) -> Or<T, Self, P>
    where
        Self: Sized,
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
    {
        MapErr {
            parser: self,
            mapping,
            marker: PhantomData,
        }
    }
}

impl<T: ParseStream, P: Parser<T> + Sealed> StandardExtension<T> for P {}

fn just_on_boundary(item: usize, bound: Bound<&usize>) -> bool {
    match bound {
        Bound::Included(e) => item == *e,
        Bound::Excluded(e) => item + 1 == *e,
        Bound::Unbounded => false,
    }
}

#[derive(Debug, Clone)]
pub enum Either<T0, T1> {
    First(T0),
    Last(T1),
}
