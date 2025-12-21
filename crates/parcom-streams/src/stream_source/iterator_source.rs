use std::marker::PhantomData;

use parcom_core::Never;
use parcom_streams_core::{BufferWriter, StreamControl, StreamSource};

#[derive(Debug)]
pub struct IteratorSource<I, T>
where
    T: Clone,
    I::Item: AsRef<[T]>,
    I: IntoIterator,
{
    iter: I::IntoIter,
    _phantom: PhantomData<fn() -> T>,
}

impl<I, T> IteratorSource<I, T>
where
    T: Clone,
    <I as IntoIterator>::Item: AsRef<[T]>,
    I: IntoIterator,
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
    T: Clone,
    I::Item: AsRef<[T]>,
    I: IntoIterator,
{
    type Item = T;
    type Error = Never;

    type Next<'a, C>
        = std::future::Ready<C::Result>
    where
        I: 'a,
        T: 'a,
        C: 'a + StreamControl<Item = Self::Item, Error = Self::Error>;

    fn next<'a, C>(&'a mut self, control: C, _size_hint: usize) -> Self::Next<'a, C>
    where
        C: 'a + StreamControl<Item = Self::Item, Error = Self::Error>,
    {
        let Some(node) = self.iter.next() else {
            return std::future::ready(control.finish());
        };

        let seg = node.as_ref();
        let mut req = control.request_writer(seg.len());

        for item in seg {
            let _ = req.push(item.clone());
        }

        std::future::ready(req.advance())
    }
}
