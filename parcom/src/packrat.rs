pub mod boundcached;
pub mod cached;

use crate::{internal::Sealed, ParseStream, Parser};

use self::{boundcached::BoundCached, cached::Cached};

pub trait BindStream: ParseStream {
    /// Bind a value to a specific location. When the stream advances, the value may be dropped.
    fn bind<T>(self, index: usize, value: T) -> Self;
}

pub trait PackratExtension<S: ParseStream>: Parser<S> + Sealed
where
    Self::Error: Clone,
    Self::Output: Clone,
{
    /// The input must always be the same stream.
    /// If a different stream is provided as input, it will produce incorrect results.
    fn cached(self) -> Cached<S, Self>
    where
        Self: Sized,
    {
        Cached::new(self)
    }

    /// The input must always be the same stream.
    /// If a different stream is provided as input, it will produce incorrect results.
    fn bound_cached(self) -> BoundCached<S, Self>
    where
        Self: Sized,
        S: BindStream,
        S::Location: Clone,
    {
        BoundCached::new(self)
    }
}

impl<S: ParseStream, P: Parser<S> + Sealed> PackratExtension<S> for P
where
    Self::Error: Clone,
    Self::Output: Clone,
{
}
