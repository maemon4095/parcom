use crate::future::pin_option::{PinOption, PinOptionProj};
use pin_project::pin_project;
use std::{future::Future, task::Poll};

#[pin_project]
pub struct OptionFuture<F: Future> {
    #[pin]
    fut: PinOption<F>,
}

impl<F: Future> OptionFuture<F> {
    pub fn some(fut: F) -> Self {
        Self {
            fut: PinOption::Some(fut),
        }
    }

    pub fn none() -> Self {
        Self {
            fut: PinOption::None,
        }
    }
}

impl<F: Future> Future for OptionFuture<F> {
    type Output = Option<F::Output>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.fut.project() {
            PinOptionProj::Some(fut) => fut.poll(cx).map(Some),
            PinOptionProj::None => Poll::Ready(None),
        }
    }
}
