use super::{GenericStream, Node};
use crate::{
    generic_stream::GenericStreamLoader,
    util::pin_option::{PinOption, PinOptionProj},
};
use parcom_streams_core::{StreamDriver, StreamDriverSession, StreamSource};
use pin_project::pin_project;
use std::{future::Future, task::Poll};

#[pin_project(!Unpin)]
pub struct Advance<'a, T, S, D>
where
    T: 'a,
    S: 'a + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'a,
{
    // WARN: DO NOT CHANGE FIELD ORDER!! `fut` must be dropped before `stream` is dropped.
    #[pin]
    fut: PinOption<<D::Session as StreamDriverSession>::WaitForAppendData<'a>>,

    stream: Option<GenericStream<'a, T, S, D>>,
    remaining: usize,
}

impl<'a, T, S, D> Advance<'a, T, S, D>
where
    T: 'a,
    S: 'a + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'a,
{
    pub(super) fn new(stream: GenericStream<'a, T, S, D>, delta: usize) -> Self {
        Self {
            stream: Some(stream),
            remaining: delta,
            fut: PinOption::None,
        }
    }
}

impl<'a, T, S, D> Future for Advance<'a, T, S, D>
where
    T: 'a,
    S: 'a + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'a,
{
    type Output = Result<GenericStream<'a, T, S, D>, S::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        if let PinOptionProj::Some(fut) = this.fut.as_mut().project() {
            std::task::ready!(fut.poll(cx))?;
            this.fut.set(PinOption::None);
        }

        let stream = this.stream.as_mut().unwrap();

        let mut current_ptr = stream.head_ptr.clone();
        let mut current_start = stream.head_start;
        let mut remaining = *this.remaining;

        loop {
            let next_ptr = match &*current_ptr {
                Node::Head(next) => next.get(),
                Node::Intermediate(node) => {
                    let len = node.buf.len() - current_start;
                    if remaining < len {
                        let mut stream = this.stream.take().unwrap();
                        let head_start = remaining + current_start;
                        let newly_examined = node.set_examined(head_start);
                        stream.head_ptr = current_ptr;
                        stream.head_start = head_start;

                        if newly_examined > 0 {
                            stream.driver_session.notify_data_examined(newly_examined);
                        }

                        return Poll::Ready(Ok(stream));
                    }

                    remaining -= len;
                    current_start = node.buf.len();

                    let newly_examined = node.set_examined(node.buf.len());
                    if newly_examined > 0 {
                        stream.driver_session.notify_data_examined(newly_examined);
                    }

                    node.next.get()
                }
                Node::Sentinel => {
                    let mut stream = this.stream.take().unwrap();
                    stream.head_ptr = current_ptr;

                    if remaining > 0 {
                        panic!("`delta` run out of stream.")
                    }

                    return Poll::Ready(Ok(stream));
                }
            };

            let Some(next_ptr) = next_ptr else {
                let stream = this.stream.as_mut().unwrap();
                stream.head_ptr = current_ptr;
                stream.head_start = current_start;
                *this.remaining = remaining;
                let fut = unsafe {
                    let fut = stream.driver_session.wait_for_append_data(0);
                    // SAFETY: `Advance`は`Unpin`であり`poll`が呼ばれる時点でピン止めされているため、`driver_session`よりも長い期間生存することはない。
                    std::mem::transmute(fut)
                };
                this.fut.set(PinOption::Some(fut));
                cx.waker().wake_by_ref();
                return Poll::Pending;
            };

            current_ptr = next_ptr.clone();
            current_start = 0;
        }
    }
}
