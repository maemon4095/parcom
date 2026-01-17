mod predicate;

pub mod extension;

pub use extension::ObservableExtensions;
pub use predicate::Predicate;

pub trait IObservable<T> {
    type Subscription<U: IObserver<T>>;
    fn subscribe<U: IObserver<T>>(&self, observer: U) -> Self::Subscription<U>;
}

pub trait IObserver<T> {
    fn next(&mut self, item: T);
}
