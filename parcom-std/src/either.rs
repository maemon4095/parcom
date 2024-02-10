use parcom_core::{ShouldNever, ShouldNeverExtension};
use std::fmt::Debug;

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

    pub fn unify<T>(self) -> T
    where
        T0: Into<T>,
        T1: Into<T>,
    {
        match self {
            Either::First(e) => e.into(),
            Either::Last(e) => e.into(),
        }
    }
}

unsafe impl<T0: ShouldNever, T1: ShouldNever> ShouldNever for Either<T0, T1> {}
