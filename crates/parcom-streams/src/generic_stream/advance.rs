use super::{GenericStream, InnerGenericStream};
use parcom_streams_core::StreamSource;
use pin_project::pin_project;
use std::{future::Future, pin::Pin, task::Poll};

#[pin_project(!Unpin)]
pub struct Advance<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    #[pin]
    state: AdvanceState<T, S>,
}

impl<T, S> Advance<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    pub(super) fn new(stream: Box<InnerGenericStream<T, S>>, delta: usize) -> Self {
        Self {
            state: AdvanceState::Initial {
                stream: Some(stream),
                delta,
            },
        }
    }
}

#[pin_project(project = AdvanceStateProj)]
enum AdvanceState<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    Initial {
        stream: Option<Box<InnerGenericStream<T, S>>>,
        delta: usize,
    },
    Loading {
        #[pin]
        fut: Pin<Box<dyn Future<Output = Result<GenericStream<T, S>, S::Error>>>>,
    },
}

impl<T, S> Future for Advance<T, S>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    type Output = Result<GenericStream<T, S>, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut state = self.project().state;
        match state.as_mut().project() {
            AdvanceStateProj::Initial { stream, delta } => {
                let Some(mut stream) = stream.take() else {
                    panic!("`poll` after completed")
                };

                match stream.advance_already_loaded(*delta) {
                    Ok(_) => Poll::Ready(Ok(GenericStream { inner: stream })),
                    Err(remain) => {
                        let fut = advance_load(stream, remain);
                        state.set(AdvanceState::Loading { fut: Box::pin(fut) });
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            AdvanceStateProj::Loading { fut } => fut.poll(cx),
        }
    }
}

async fn advance_load<T, S>(
    mut stream: Box<InnerGenericStream<T, S>>,
    mut remain: usize,
) -> Result<GenericStream<T, S>, S::Error>
where
    T: 'static + Default,
    S: 'static + StreamSource<Segment = [T]>,
{
    debug_assert!(remain > 0);

    loop {
        let written = stream.load(remain).await?;

        if written == 0 || remain <= written {
            break;
        }

        stream.advance_already_loaded_all();
        remain -= written;
    }

    if let Err(_) = stream.advance_already_loaded(remain) {
        panic!("`delta` out of stream")
    };

    Ok(GenericStream { inner: stream })
}
