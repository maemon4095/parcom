use std::{future::Future, marker::PhantomData, pin::Pin, task::Poll};

use parcom_streams_core::{BufferRequest, StreamControl, StreamSource};

pub struct Utf8Validator<S: StreamSource<Segment = [u8]>> {
    buffered: usize,
    rest: [u8; 3],
    source: S,
}

impl<S: StreamSource<Segment = [u8]>> Utf8Validator<S> {
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

impl<S: StreamSource<Segment = [u8]>> StreamSource for Utf8Validator<S> {
    type Segment = [u8];
    type Error = Utf8ValidationError<S::Error>;

    type Next<'a, C>
        = Next<'a, S, C>
    where
        Self: 'a,
        C: StreamControl<Segment = Self::Segment, Error = Self::Error>;

    fn next<C>(&mut self, control: C, size_hint: usize) -> Self::Next<'_, C>
    where
        C: StreamControl<Segment = Self::Segment, Error = Self::Error>,
    {
        let control = Control::<S, C> {
            buffered: self.buffered,
            rest: self.rest,
            control,
            _phantom: PhantomData,
        };

        let fut = self.source.next(control, size_hint);

        Next {
            buffered: &mut self.buffered,
            rest: &mut self.rest,
            fut,
        }
    }
}

pub struct Next<
    'a,
    S: StreamSource<Segment = [u8]> + 'a,
    C: StreamControl<Segment = [u8], Error = Utf8ValidationError<S::Error>>,
> {
    buffered: &'a mut usize,
    rest: &'a mut [u8; 3],
    fut: S::Next<'a, Control<S, C>>,
}

impl<
        'a,
        S: StreamSource<Segment = [u8]>,
        C: StreamControl<Segment = [u8], Error = Utf8ValidationError<S::Error>>,
    > Future for Next<'a, S, C>
{
    type Output = C::Response;

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

struct Control<S: StreamSource<Segment = [u8]>, C: StreamControl> {
    buffered: usize,
    rest: [u8; 3],
    control: C,
    _phantom: PhantomData<fn(S) -> S>,
}

impl<S, C> StreamControl for Control<S, C>
where
    S: StreamSource<Segment = [u8]>,
    C: StreamControl<Segment = [u8], Error = Utf8ValidationError<S::Error>>,
{
    type Segment = [u8];
    type Error = S::Error;
    type Response = Response<S, C>;
    type Request = Request<S, C>;

    fn request_buffer(self, min_size: usize) -> Self::Request {
        let mut req = self.control.request_buffer(self.buffered + min_size);

        req.buffer()[..self.buffered].copy_from_slice(&self.rest[..self.buffered]);

        Request {
            buffered: self.buffered,
            req,
            _phantom: PhantomData,
        }
    }

    fn cancel(self, err: S::Error) -> Self::Response {
        let res = self.control.cancel(Utf8ValidationError::Inner(err));
        Response {
            inner: InnerResponse::Err { res },
            _phantom: PhantomData,
        }
    }

    fn finish(self) -> Self::Response {
        let res = self.control.finish();
        Response {
            inner: InnerResponse::Finish { res },
            _phantom: PhantomData,
        }
    }
}

struct Request<S, C>
where
    S: StreamSource<Segment = [u8]>,
    C: StreamControl<Segment = [u8], Error = Utf8ValidationError<S::Error>>,
{
    buffered: usize,
    req: C::Request,
    _phantom: PhantomData<fn(S) -> S>,
}

impl<S, C> BufferRequest for Request<S, C>
where
    S: StreamSource<Segment = [u8]>,
    C: StreamControl<Segment = [u8], Error = Utf8ValidationError<S::Error>>,
{
    type Control = Control<S, C>;

    fn buffer(&mut self) -> &mut [u8] {
        &mut self.req.buffer()[self.buffered..]
    }

    fn advance(mut self, written: usize) -> Response<S, C> {
        let total = self.buffered + written;
        let buf = &self.req.buffer()[..total];

        let valid_len = match std::str::from_utf8(buf) {
            Ok(_) => total,
            Err(e) => e.valid_up_to(),
        };

        let invalids = &buf[valid_len..];

        let inner = if invalids.len() > 3 {
            let res = self.req.cancel(Utf8ValidationError::InvalidSequence);
            InnerResponse::Err { res }
        } else {
            let mut rest = [0; 3];
            rest[..invalids.len()].copy_from_slice(invalids);

            InnerResponse::Advance {
                buffered: invalids.len(),
                rest,
                res: self.req.advance(valid_len),
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

struct Response<S: StreamSource<Segment = [u8]>, C: StreamControl>
where
    S: StreamSource<Segment = [u8]>,
    C: StreamControl<Segment = [u8], Error = Utf8ValidationError<S::Error>>,
{
    inner: InnerResponse<C>,
    _phantom: PhantomData<fn(S) -> S>,
}

enum InnerResponse<C: StreamControl>
where
    C: StreamControl<Segment = [u8]>,
{
    Advance {
        buffered: usize,
        rest: [u8; 3],
        res: C::Response,
    },
    Finish {
        res: C::Response,
    },
    Err {
        res: C::Response,
    },
}

#[cfg(test)]

mod test {
    use super::*;
    use crate::{
        stream_control::vec_control::VecControl, stream_source::iterator_source::IteratorSource,
    };

    #[test]
    fn test_valid() {
        // "a": 1 byte
        // "Α": 2 byte
        // "あ": 3 byte
        // "😀": 4 byte
        //
        // バイト長ごとに隣接したパターンをすべて確認する。
        // 隣接パターン: 11, 12, 13, 14, 21, 22, 23, 24, 31, 32, 33, 34, 41, 42, 43, 44
        // 上の隣接パターンをすべてもつ列: 11213142232433441 をテストに使う。
        let text = "aaΑaあa😀ΑΑあΑ😀ああ😀😀a";
        let bin = text.as_bytes();

        for i in 0..bin.len() {
            let (l, r) = bin.split_at(i);
            let src = IteratorSource::new([l, r]);
            let mut src = Utf8Validator::new(src);
            let mut buf = Vec::new();

            loop {
                let control = VecControl::new(buf);

                let res = pollster::block_on(src.next(control, 0));

                match res {
                    crate::stream_control::Response::Advance(v) => {
                        buf = v;
                        let r = std::str::from_utf8(&buf).unwrap();
                        assert!(text.starts_with(r));
                    }
                    crate::stream_control::Response::Finish(v) => {
                        buf = v;
                        let r = std::str::from_utf8(&buf).unwrap();
                        assert!(text.starts_with(r));
                        assert_eq!(r, text);
                        break;
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
