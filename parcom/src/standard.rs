pub mod binary_expr;
pub mod parse;
pub mod parser;

use crate::ShouldNever;
use std::{fmt::Debug, ops::Bound};

pub use parser::ParserExtension;

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

impl<T0, T1> Either<T0, T1> {
    pub fn unwrap_first(self) -> T0
    where
        T1: Debug,
    {
        match self {
            Either::First(e) => e,
            Either::Last(e) => {
                panic!(
                    "`Either::unwrap_first` was called on an `Either::Last({:?})`.",
                    e
                )
            }
        }
    }

    pub fn unwrap_last(self) -> T1
    where
        T0: Debug,
    {
        match self {
            Either::First(e) => panic!(
                "`Either::unwrap_last` was called on an `Either::First({:?})`.",
                e
            ),
            Either::Last(e) => e,
        }
    }

    pub fn always_first(self) -> T0
    where
        T1: ShouldNever,
    {
        match self {
            Either::First(e) => e,
            Either::Last(e) => e.never(),
        }
    }

    pub fn always_last(self) -> T1
    where
        T0: ShouldNever,
    {
        match self {
            Either::First(e) => e.never(),
            Either::Last(e) => e,
        }
    }
}

impl<T0: ShouldNever, T1: ShouldNever> ShouldNever for Either<T0, T1> {}
