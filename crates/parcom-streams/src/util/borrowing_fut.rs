use std::{future::Future, marker::PhantomData, task::Poll};

use pin_project::pin_project;

use crate::util::getter::Getter;

#[pin_project]
pub struct BorrowingFuture<'a, T, F>
where
    F: Future,
{
    #[pin]
    state: BorrowingFutureState<T, F>,
    _marker: PhantomData<&'a mut T>,
}
impl<'a, T, F: Future> BorrowingFuture<'a, T, F>
where
    F: Future,
{
    /// 可変参照はユニークでなければならないため、`F::Output` は `receiver` への参照を含んではならない。
    pub unsafe fn new(receiver: &'a mut T, f: impl Getter<&'a mut T, F>) -> Self {
        let ptr: *mut T = receiver;
        let fut = f.get(receiver);

        Self {
            state: BorrowingFutureState::Polling { receiver: ptr, fut },
            _marker: PhantomData,
        }
    }
}

impl<'a, T, F> Future for BorrowingFuture<'a, T, F>
where
    F: Future,
{
    type Output = (&'a mut T, F::Output);

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        match this.state.as_mut().project() {
            BorrowingFutureStateProj::Polling { receiver, fut } => {
                let result = std::task::ready!(fut.poll(cx));
                let receiver: &'a mut _ = unsafe { &mut **receiver };
                this.state.set(BorrowingFutureState::Done);
                Poll::Ready((receiver, result))
            }
            BorrowingFutureStateProj::Done => panic!("polled after ready"),
        }
    }
}

#[pin_project(project = BorrowingFutureStateProj)]
enum BorrowingFutureState<T, F: Future> {
    Polling {
        receiver: *mut T,
        #[pin]
        fut: F,
    },
    Done,
}
