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

pub enum Utf8ValidationError<S: StreamSource<Segment = [u8]>> {
    InvalidSequence,
    Inner(S::Error),
}

impl<S: StreamSource<Segment = [u8]>> StreamSource for Utf8Validator<S> {
    type Segment = [u8];
    type Error = Utf8ValidationError<S>;

    type Next<'a, C: StreamControl<Self>>
        = Next<'a, S, C>
    where
        Self: 'a;

    fn next<C: StreamControl<Self>>(&mut self, control: C, size_hint: usize) -> Self::Next<'_, C> {
        let writer = Control::<S, C> {
            buffered: self.buffered,
            rest: self.rest,
            control,
            mark: PhantomData,
        };

        let fut = self.source.next(writer, size_hint);

        Next {
            buffered: &mut self.buffered,
            rest: &mut self.rest,
            fut,
        }
    }
}

pub struct Next<'a, S: StreamSource<Segment = [u8]> + 'a, W: StreamControl<Utf8Validator<S>>> {
    buffered: &'a mut usize,
    rest: &'a mut [u8; 3],
    fut: S::Next<'a, Control<S, W>>,
}

impl<'a, S: StreamSource<Segment = [u8]>, W: StreamControl<Utf8Validator<S>>> Future
    for Next<'a, S, W>
{
    type Output = W::Response;

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

struct Control<S: StreamSource<Segment = [u8]>, W: StreamControl<Utf8Validator<S>>> {
    buffered: usize,
    rest: [u8; 3],
    control: W,
    mark: PhantomData<fn(S) -> S>,
}

impl<S: StreamSource<Segment = [u8]>, C: StreamControl<Utf8Validator<S>>> StreamControl<S>
    for Control<S, C>
{
    type Response = Response<S, C>;
    type Request = Request<S, C>;

    fn request_buffer(self, min_size: usize) -> Self::Request {
        let mut req = self.control.request_buffer(self.buffered + min_size);

        req.buffer()[..self.buffered].copy_from_slice(&self.rest);

        Request {
            buffered: self.buffered,
            req,
        }
    }

    fn cancel(self, err: S::Error) -> Self::Response {
        let res = self.control.cancel(Utf8ValidationError::Inner(err));
        Response {
            inner: InnerResponse::Err { res },
        }
    }

    fn finish(self) -> Self::Response {
        let res = self.control.finish();
        Response {
            inner: InnerResponse::Finish { res },
        }
    }
}

struct Request<S: StreamSource<Segment = [u8]>, W: StreamControl<Utf8Validator<S>>> {
    buffered: usize,
    req: W::Request,
}

impl<S: StreamSource<Segment = [u8]>, W: StreamControl<Utf8Validator<S>>> BufferRequest<S>
    for Request<S, W>
{
    type Response = Response<S, W>;

    fn buffer(&mut self) -> &mut [u8] {
        &mut self.req.buffer()[self.buffered..]
    }

    fn advance(mut self, written: usize) -> Self::Response {
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

        Response { inner }
    }

    fn cancel(self, err: S::Error) -> Self::Response {
        let res = self.req.cancel(Utf8ValidationError::Inner(err));

        Response {
            inner: InnerResponse::Err { res },
        }
    }
}

struct Response<S: StreamSource<Segment = [u8]>, W: StreamControl<Utf8Validator<S>>> {
    inner: InnerResponse<S, W>,
}

enum InnerResponse<S: StreamSource<Segment = [u8]>, W: StreamControl<Utf8Validator<S>>> {
    Advance {
        buffered: usize,
        rest: [u8; 3],
        res: W::Response,
    },
    Finish {
        res: W::Response,
    },
    Err {
        res: W::Response,
    },
}

#[cfg(test)]

mod test {
    use super::*;
    use crate::stream_source::iterator_source::IteratorSource;

    #[test]
    fn test_valid() {
        // "a": 1 byte
        // "Α": 2 byte
        // "あ": 3 byte
        // "😀": 4 byte

        // 11, 12, 13, 14, 21, 22, 23, 24, 31, 32, 33, 34, 41, 42, 43, 44
        let text = "aaaΑaあa😀";
        let bin = text.as_bytes();

        for i in 0..bin.len() {
            let (l, r) = bin.split_at(i);
            let src = IteratorSource::new([l, r]);

            let src = Utf8Validator::new(src);
        }
    }
}
