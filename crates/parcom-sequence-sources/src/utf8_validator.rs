use parcom_sequence_core::{BufferWriter, SequenceControl, SequenceSource};
use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
    task::Poll,
};

pub struct Utf8Validator<S: SequenceSource<Item = u8>> {
    buffered: usize,
    rest: [u8; 3],
    source: S,
}

impl<S: SequenceSource<Item = u8>> Utf8Validator<S> {
    pub fn new(source: S) -> Self {
        Self {
            buffered: 0,
            rest: [0; 3],
            source,
        }
    }
}

pub enum Utf8ValidationError<E> {
    InvalidSequence,
    Inner(E),
}

impl<S: SequenceSource<Item = u8>> SequenceSource for Utf8Validator<S> {
    type Item = u8;
    type Error = Utf8ValidationError<S::Error>;

    type Next<'a, C>
        = Next<'a, S, C>
    where
        Self: 'a,
        C: 'a + SequenceControl<Item = Self::Item, Error = Self::Error>;

    fn next<'a, C>(&'a mut self, control: C, size_hint: usize) -> Self::Next<'a, C>
    where
        C: 'a + SequenceControl<Item = Self::Item, Error = Self::Error>,
    {
        let control = Control::<S, C> {
            buffered: self.buffered,
            rest: self.rest,
            control,
            _phantom: PhantomData,
        };

        let fut = self.source.next(control, size_hint).into_future();

        Next {
            buffered: &mut self.buffered,
            rest: &mut self.rest,
            fut,
        }
    }
}

pub struct Next<
    'a,
    S: SequenceSource<Item = u8> + 'a,
    C: 'a + SequenceControl<Item = u8, Error = Utf8ValidationError<S::Error>>,
> {
    buffered: &'a mut usize,
    rest: &'a mut [u8; 3],
    fut: <S::Next<'a, Control<S, C>> as IntoFuture>::IntoFuture,
}

impl<'a, S, C> Future for Next<'a, S, C>
where
    S: SequenceSource<Item = u8>,
    C: SequenceControl<Item = u8, Error = Utf8ValidationError<S::Error>>,
{
    type Output = C::Result;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let fut = unsafe { Pin::new_unchecked(&mut this.fut) };

        match fut.poll(cx) {
            Poll::Ready(res) => match res.inner {
                InnerResponse::Advance {
                    buffered,
                    rest,
                    res,
                } => {
                    *this.buffered = buffered;
                    *this.rest = rest;

                    Poll::Ready(res)
                }
                InnerResponse::Err { res } => Poll::Ready(res),
                InnerResponse::Finish { res } => Poll::Ready(res),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

struct Control<S: SequenceSource<Item = u8>, C: SequenceControl> {
    buffered: usize,
    rest: [u8; 3],
    control: C,
    _phantom: PhantomData<fn(S) -> S>,
}

impl<S, C> SequenceControl for Control<S, C>
where
    S: SequenceSource<Item = u8>,
    C: SequenceControl<Item = u8, Error = Utf8ValidationError<S::Error>>,
{
    type Item = u8;
    type Error = S::Error;
    type Result = Response<S, C>;
    type Writer = Writer<S, C>;

    fn request_writer(self, min_capacity: usize) -> Self::Writer {
        let mut req = self.control.request_writer(self.buffered + min_capacity);
        let dst = req.as_mut_ptr();
        let src = self.rest.as_ptr();

        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, self.buffered);
            req.set_len(self.buffered);
        }

        Writer {
            buffered: self.buffered,
            req,
            _phantom: PhantomData,
        }
    }

    fn cancel(self, err: S::Error) -> Self::Result {
        let res = self.control.cancel(Utf8ValidationError::Inner(err));
        Response {
            inner: InnerResponse::Err { res },
            _phantom: PhantomData,
        }
    }

    fn finish(self) -> Self::Result {
        let res = self.control.finish();
        Response {
            inner: InnerResponse::Finish { res },
            _phantom: PhantomData,
        }
    }
}

struct Writer<S, C>
where
    S: SequenceSource<Item = u8>,
    C: SequenceControl<Item = u8, Error = Utf8ValidationError<S::Error>>,
{
    buffered: usize,
    req: C::Writer,
    _phantom: PhantomData<fn(S) -> S>,
}

impl<S, C> BufferWriter for Writer<S, C>
where
    S: SequenceSource<Item = u8>,
    C: SequenceControl<Item = u8, Error = Utf8ValidationError<S::Error>>,
{
    type Segment = str;
    type Item = u8;
    type Error = S::Error;
    type Result = Response<S, C>;

    fn capacity(&self) -> usize {
        self.req.capacity() - self.buffered
    }

    fn len(&self) -> usize {
        self.req.len() - self.buffered
    }

    fn as_ptr(&self) -> *const Self::Item {
        unsafe { self.req.as_ptr().add(self.buffered) }
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        unsafe { self.req.as_mut_ptr().add(self.buffered) }
    }

    unsafe fn set_len(&mut self, len: usize) {
        self.req.set_len(len + self.buffered);
    }

    fn advance(mut self) -> Response<S, C> {
        let buf = self.req.as_slice();

        let valid_len = match std::str::from_utf8(buf) {
            Ok(_) => buf.len(),
            Err(e) => e.valid_up_to(),
        };

        let invalids = &buf[valid_len..];

        let inner = if invalids.len() > 3 {
            let res = self.req.cancel(Utf8ValidationError::InvalidSequence);
            InnerResponse::Err { res }
        } else {
            let mut rest = [0; 3];
            rest[..invalids.len()].copy_from_slice(invalids);

            let buffered = invalids.len();
            self.req.shrink_to(valid_len);

            InnerResponse::Advance {
                buffered,
                rest,
                res: self.req.advance(),
            }
        };

        Response {
            inner,
            _phantom: PhantomData,
        }
    }

    fn cancel(self, err: S::Error) -> Response<S, C> {
        let res = self.req.cancel(Utf8ValidationError::Inner(err));

        Response {
            inner: InnerResponse::Err { res },
            _phantom: PhantomData,
        }
    }
}

struct Response<S: SequenceSource<Item = u8>, C: SequenceControl>
where
    S: SequenceSource<Item = u8>,
    C: SequenceControl<Item = u8, Error = Utf8ValidationError<S::Error>>,
{
    inner: InnerResponse<C>,
    _phantom: PhantomData<fn(S) -> S>,
}

enum InnerResponse<C: SequenceControl>
where
    C: SequenceControl<Item = u8>,
{
    Advance {
        buffered: usize,
        rest: [u8; 3],
        res: C::Result,
    },
    Finish {
        res: C::Result,
    },
    Err {
        res: C::Result,
    },
}

#[cfg(test)]

mod test {
    // TODO: tests/配下に移動
    // use super::*;
    // use crate::{control::vec_control::VecControl, source::iterator_source::IteratorSource};

    // #[test]
    // fn test_valid() {
    //     // "a": 1 byte
    //     // "Α": 2 byte
    //     // "あ": 3 byte
    //     // "😀": 4 byte
    //     //
    //     // バイト長ごとに隣接したパターンをすべて確認する。
    //     // 隣接パターン: 11, 12, 13, 14, 21, 22, 23, 24, 31, 32, 33, 34, 41, 42, 43, 44
    //     // 上の隣接パターンをすべてもつ列: 11213142232433441 をテストに使う。
    //     let text = "aaΑaあa😀ΑΑあΑ😀ああ😀😀a";
    //     let bin = text.as_bytes();

    //     for i in 0..bin.len() {
    //         let (l, r) = bin.split_at(i);
    //         let src = IteratorSource::new([l, r]);
    //         let mut src = Utf8Validator::new(src);
    //         let mut buf = Vec::new();

    //         loop {
    //             let control = VecControl::new(&mut buf);

    //             let res = pollster::block_on(src.next(control, 0));

    //             match res {
    //                 crate::control::Response::Advance(_) => {
    //                     let r = std::str::from_utf8(&buf).unwrap();
    //                     assert!(text.starts_with(r));
    //                 }
    //                 crate::control::Response::Finish(_) => {
    //                     let r = std::str::from_utf8(&buf).unwrap();
    //                     assert!(text.starts_with(r));
    //                     assert_eq!(r, text);
    //                     break;
    //                 }
    //                 _ => unreachable!(),
    //             }
    //         }
    //     }
    // }
}
