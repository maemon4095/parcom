use std::marker::PhantomData;

use parcom_core::Never;
use parcom_streams_core::{BufferRequest, StreamControl, StreamSource};

pub struct IteratorSource<I, T>
where
    I::Item: AsRef<[T]>,
    I: IntoIterator,
    T: Copy,
{
    iter: I::IntoIter,
    _phantom: PhantomData<fn() -> T>,
}

impl<I, T> IteratorSource<I, T>
where
    <I as IntoIterator>::Item: AsRef<[T]>,
    I: IntoIterator,
    T: Copy,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.into_iter(),
            _phantom: PhantomData,
        }
    }
}

impl<I, T> StreamSource for IteratorSource<I, T>
where
    I::Item: AsRef<[T]>,
    I: IntoIterator,
    T: Copy,
{
    type Segment = [T];
    type Error = Never;

    type Next<'a, C: StreamControl<Self>>
        = std::future::Ready<C::Response>
    where
        I: 'a,
        T: 'a;

    fn next<C: StreamControl<Self>>(&mut self, control: C, _size_hint: usize) -> Self::Next<'_, C> {
        let Some(node) = self.iter.next() else {
            return std::future::ready(control.finish());
        };

        let seg = node.as_ref();
        let mut req = control.request_buffer(seg.len());

        req.buffer()[..seg.len()].copy_from_slice(seg);
        std::future::ready(req.advance(seg.len()))
    }
}
