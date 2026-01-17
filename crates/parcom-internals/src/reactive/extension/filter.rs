use std::{marker::PhantomData, sync::Arc};

use crate::reactive::{IObservable, IObserver, Predicate};

pub struct Filter<T, O: IObservable<T>, P: Predicate<T>> {
    observable: O,
    predicate: Arc<P>,
    _phantom: PhantomData<T>,
}

impl<T, O: IObservable<T>, P: Predicate<T>> Filter<T, O, P> {
    pub(super) fn new(observable: O, predicate: P) -> Self {
        Self {
            observable,
            predicate: Arc::new(predicate),
            _phantom: PhantomData,
        }
    }
}

pub struct Observer<T, O: IObserver<T>, P: Predicate<T>> {
    observer: O,
    predicate: Arc<P>,
    _phantom: PhantomData<T>,
}

impl<T, O: IObserver<T>, P: Predicate<T>> IObserver<T> for Observer<T, O, P> {
    fn next(&mut self, item: T) {
        if self.predicate.test(&item) {
            self.observer.next(item);
        }
    }
}

pub struct Subscription<T, O: IObservable<T>, P: Predicate<T>, U: IObserver<T>>(
    O::Subscription<Observer<T, U, P>>,
);

impl<T, O: IObservable<T>, P: Predicate<T>> IObservable<T> for Filter<T, O, P> {
    type Subscription<U: IObserver<T>> = Subscription<T, O, P, U>;

    fn subscribe<U: IObserver<T>>(&self, observer: U) -> Self::Subscription<U> {
        let o = Observer {
            observer,
            predicate: Arc::clone(&self.predicate),
            _phantom: PhantomData,
        };
        let sub = self.observable.subscribe(o);
        Subscription(sub)
    }
}
