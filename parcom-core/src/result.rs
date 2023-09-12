use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Result<O, E, F = crate::Never> {
    Ok(O),
    Err(E),
    Fault(F),
}
use crate::ShouldNever;

use self::Result::*;

impl<O, E, F> Result<O, E, F> {
    pub fn map<T>(self, f: impl FnOnce(O) -> T) -> Result<T, E, F> {
        match self {
            Ok(v) => Ok(f(v)),
            Err(e) => Err(e),
            Fault(e) => Fault(e),
        }
    }

    pub fn map_err<T>(self, f: impl FnOnce(E) -> T) -> Result<O, T, F> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(f(e)),
            Fault(e) => Fault(e),
        }
    }

    pub fn map_fault<T>(self, f: impl FnOnce(F) -> T) -> Result<O, E, T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
            Fault(e) => Fault(f(e)),
        }
    }

    pub fn and_then<T>(self, f: impl FnOnce(O) -> Result<T, E, F>) -> Result<T, E, F> {
        match self {
            Ok(v) => match f(v) {
                Ok(v) => Ok(v),
                Err(e) => Err(e),
                Fault(e) => Fault(e),
            },
            Err(e) => Err(e),
            Fault(e) => Fault(e),
        }
    }

    pub fn ok(self) -> Option<O> {
        match self {
            Ok(v) => Some(v),
            _ => None,
        }
    }

    pub fn err(self) -> Option<E> {
        match self {
            Err(e) => Some(e),
            _ => None,
        }
    }

    pub fn fault(self) -> Option<F> {
        match self {
            Fault(e) => Some(e),
            _ => None,
        }
    }

    pub fn never_fault(self) -> Result<O, E>
    where
        F: ShouldNever,
    {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
            Fault(e) => Fault(e.never()),
        }
    }

    pub fn unwrap(self) -> O
    where
        E: Debug,
        F: Debug,
    {
        match self {
            Ok(v) => v,
            Err(e) => panic!("called Result::unwrap on an Fail value; Error: {:?}.", e),
            Fault(e) => panic!("called Result::unwrap on an Fault value; Error: {:?}.", e),
        }
    }

    pub fn unwrap_err(self) -> E
    where
        O: Debug,
        F: Debug,
    {
        match self {
            Ok(v) => panic!("called Result::unwrap_err on an Ok value; Output: {:?}.", v),
            Fault(e) => panic!(
                "called Result::unwrap_err on an Fault value; Output: {:?}.",
                e
            ),
            Err(e) => e,
        }
    }

    pub fn unwrap_fault(self) -> F
    where
        O: Debug,
        E: Debug,
    {
        match self {
            Ok(v) => panic!("called Result::unwrap_err on an Ok value; Output: {:?}.", v),
            Err(e) => panic!(
                "called Result::unwrap_err on an Err value; Output: {:?}.",
                e
            ),
            Fault(e) => e,
        }
    }

    pub fn as_ref(&self) -> Result<&O, &E, &F> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
            Fault(e) => Fault(e),
        }
    }
    pub fn as_mut(&mut self) -> Result<&mut O, &mut E, &mut F> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
            Fault(e) => Fault(e),
        }
    }
}
