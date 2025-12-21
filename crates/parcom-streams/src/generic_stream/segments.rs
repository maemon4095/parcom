use crate::{
    generic_stream::{GenericStream, GenericStreamLoader, Node, NodePtr},
    util::{
        borrowing_fut::BorrowingFuture,
        getter::Getter,
        pin_option::{PinOption, PinOptionProj},
    },
};
use parcom_core::SegmentStream;
use parcom_streams_core::{StreamDriver, StreamDriverSession, StreamSource};
use pin_project::pin_project;
use std::{future::Future, marker::PhantomData, task::Poll};

pub struct Segments<'stream, 'b, T, S, D>
where
    T: 'stream,
    S: 'stream + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'stream,
{
    session: Option<&'b mut D::Session>,
    current_start: usize,
    current_node_ptr: Option<&'b NodePtr<T>>,
    _phantom: PhantomData<&'stream mut ()>,
}
impl<'stream, 'b, T, S, D> Segments<'stream, 'b, T, S, D>
where
    T: 'stream,
    S: 'stream + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'stream,
{
    pub(super) fn new(stream: &'b mut GenericStream<T, S, D>) -> Self {
        let current_start = stream.head_start;
        let (session, head_ptr) = stream.session_head_pair();
        Self {
            session: Some(session),
            current_start,
            current_node_ptr: Some(head_ptr),
            _phantom: PhantomData,
        }
    }
}

impl<'stream, 'b, T, S, D> SegmentStream for Segments<'stream, 'b, T, S, D>
where
    T: 'stream,
    S: 'stream + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'stream,
{
    type SegmentRef = &'b [T];
    type Error = S::Error;
    type Next<'a>
        = Next<'stream, 'a, 'b, T, S, D>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Self::Next<'a> {
        Next {
            fut: PinOption::None,
            iter: self,
        }
    }
}

#[pin_project(project = NextProj)]
pub struct Next<'stream, 'a, 'b, T, S, D>
where
    T: 'stream,
    S: 'stream + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'stream,
{
    iter: &'a mut Segments<'stream, 'b, T, S, D>,

    #[pin]
    fut: PinOption<
        BorrowingFuture<'b, D::Session, <D::Session as StreamDriverSession>::WaitForAppendData<'b>>,
    >,
}

impl<'stream, 'a, 'b, T, S, D> Future for Next<'stream, 'a, 'b, T, S, D>
where
    T: 'stream,
    S: 'stream + StreamSource<Item = T>,
    D: StreamDriver<GenericStreamLoader<T, S>>,
    D::Session: 'stream,
{
    type Output = Result<Option<&'b [T]>, S::Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this: NextProj<'_, 'stream, 'a, 'b, T, S, D> = self.project();
        let Some(&current_node_ptr) = this.iter.current_node_ptr.as_ref().take() else {
            return Poll::Ready(Ok(None));
        };

        let fut = this.fut.as_mut().project();

        if let PinOptionProj::Some(fut) = fut {
            let (session, result) = std::task::ready!(fut.poll(cx));
            this.iter.session = Some(session);
            this.fut.set(PinOption::None);
            result?
        };

        let next_ptr = match &**current_node_ptr {
            Node::Head(v) => v.get(),
            Node::Intermediate(v) => {
                let current_start = this.iter.current_start;
                // 返すバッファの長さが0になる場合、スキップする。
                if current_start < v.buf.len() {
                    // `current_start`を現在のノードのバッファの長さと同じにすることで、次回のpollでスキップされるようにする。
                    this.iter.current_start = v.buf.len();

                    let newly_examined = v.set_examined(v.buf.len());
                    if newly_examined > 0 {
                        this.iter
                            .session
                            .as_mut()
                            .unwrap()
                            .notify_data_examined(newly_examined);
                    }
                    return Poll::Ready(Ok(Some(&v.buf[current_start..])));
                }
                v.next.get()
            }
            Node::Sentinel => return Poll::Ready(Ok(None)),
        };

        // 現在のノードはイテレーション済み。次のノードを返す。
        let next_ptr = match next_ptr {
            Some(v) => v,
            None => {
                // 次のノードは読み込まれていないため、次のノードが追加されるまで待つ。
                let fut = unsafe {
                    BorrowingFuture::new(this.iter.session.take().unwrap(), WaitForAppendDataGetter)
                };

                // データ追加を待つ`fut`を作成している間にデータが追加されていないか確認する。
                let next_ptr = match &**current_node_ptr {
                    Node::Head(v) => v.get(),
                    Node::Intermediate(v) => v.next.get(),
                    Node::Sentinel => unreachable!(),
                };

                // `fut`を待つ前にデータが追加されていた場合はそのノードを返し、追加されていなければ`fut`を待つ。
                let Some(next_ptr) = next_ptr else {
                    this.fut.set(PinOption::Some(fut));
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                };

                next_ptr
            }
        };

        this.iter.current_node_ptr = Some(next_ptr);
        this.iter.current_start = 0;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

struct WaitForAppendDataGetter;

impl<'a, S: StreamDriverSession> Getter<&'a mut S, S::WaitForAppendData<'a>>
    for WaitForAppendDataGetter
{
    fn get(&self, value: &'a mut S) -> S::WaitForAppendData<'a> {
        value.wait_for_append_data(0)
    }
}
