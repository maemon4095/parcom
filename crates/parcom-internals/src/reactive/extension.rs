pub mod filter;

use crate::reactive::{IObservable, Predicate};

pub use filter::Filter;

pub trait ObservableExtensions<T>: Sized + IObservable<T> {
    fn filter<P: Predicate<T>>(self, predicate: P) -> Filter<T, Self, P> {
        Filter::new(self, predicate)
    }
}

impl<T, O: IObservable<T>> ObservableExtensions<T> for O {}
