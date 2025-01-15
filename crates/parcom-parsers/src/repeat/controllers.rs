use std::mem::MaybeUninit;

use parcom_core::{Never, Parser, RewindStream};

use super::{RepeatContol, RepeatContollerResult, RepeatController};

impl<S: RewindStream, P: Parser<S>> RepeatController<S, P> for usize {
    type Output = Vec<P::Output>;
    type Error = P::Error;

    async fn control(
        &self,
        mut control: RepeatContol<'_, S, P, Self>,
    ) -> RepeatContollerResult<S, P, Self> {
        let mut buf = Vec::with_capacity(*self);

        for _ in 0..(*self) {
            match control.next().await {
                Ok((v, c)) => {
                    buf.push(v);
                    control = c;
                }
                Err((e, c)) => return c.fail(e),
            }
        }

        control.done(buf)
    }
}

pub struct Const<const N: usize>;

impl<S: RewindStream, P: Parser<S>, const N: usize> RepeatController<S, P> for Const<N> {
    type Output = [P::Output; N];
    type Error = P::Error;

    async fn control(
        &self,
        mut control: RepeatContol<'_, S, P, Self>,
    ) -> RepeatContollerResult<S, P, Self> {
        let mut buf: [_; N] = std::array::from_fn(|_| MaybeUninit::uninit());

        for i in 0..N {
            match control.next().await {
                Ok((v, c)) => {
                    buf[i].write(v);
                    control = c;
                }
                Err((e, c)) => {
                    for j in 0..i {
                        unsafe {
                            buf[j].assume_init_drop();
                        }
                    }

                    return c.fail(e);
                }
            }
        }

        let result = unsafe { buf.map(|e| e.assume_init()) };
        control.done(result)
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatController<S, P> for std::ops::RangeFull {
    type Output = (Vec<P::Output>, P::Error);
    type Error = Never;

    async fn control(
        &self,
        mut control: RepeatContol<'_, S, P, Self>,
    ) -> RepeatContollerResult<S, P, Self> {
        let mut buf = Vec::new();

        loop {
            let anchor = control.anchor();
            match control.next().await {
                Ok((v, c)) => {
                    buf.push(v);
                    control = c;
                }
                Err((e, c)) => {
                    return c.done((buf, e), anchor).await;
                }
            }
        }
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatController<S, P> for std::ops::RangeTo<usize> {
    type Output = (Vec<P::Output>, Option<P::Error>);
    type Error = Never;

    async fn control(
        &self,
        mut control: RepeatContol<'_, S, P, Self>,
    ) -> RepeatContollerResult<S, P, Self> {
        let mut buf = Vec::with_capacity(self.end);

        for _ in 0..self.end {
            let anchor = control.anchor();
            match control.next().await {
                Ok((v, c)) => {
                    buf.push(v);
                    control = c;
                }
                Err((e, c)) => {
                    return c.done((buf, Some(e)), anchor).await;
                }
            }
        }

        control.done((buf, None))
    }
}

impl<S: RewindStream, P: Parser<S>> RepeatController<S, P> for std::ops::RangeToInclusive<usize> {
    type Output = (Vec<P::Output>, Option<P::Error>);
    type Error = Never;

    async fn control(
        &self,
        mut control: RepeatContol<'_, S, P, Self>,
    ) -> RepeatContollerResult<S, P, Self> {
        let mut buf = Vec::with_capacity(self.end);

        for _ in 0..=self.end {
            let anchor = control.anchor();
            match control.next().await {
                Ok((v, c)) => {
                    buf.push(v);
                    control = c;
                }
                Err((e, c)) => {
                    return c.done((buf, Some(e)), anchor).await;
                }
            }
        }

        control.done((buf, None))
    }
}
