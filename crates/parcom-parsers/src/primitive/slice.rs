use parcom_core::{Never, ParcomStream, ParseResult::*, Parser, ParserResult};

pub fn atom<T>(items: &[T]) -> Atom<'_, T>
where
    T: PartialEq,
{
    Atom { items }
}

pub fn single<'a, T>(item: &'a T) -> Single<'a, T>
where
    T: PartialEq,
{
    Single { item }
}

pub struct Atom<'a, T>
where
    T: PartialEq,
{
    items: &'a [T],
}

impl<'a, T, S> Parser<S> for Atom<'a, T>
where
    T: PartialEq,
    S: ParcomStream<Segment = [T]>,
{
    type Output = &'a [T];
    type Error = ();
    type Fault = Never;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let mut remain = self.items;
        let mut segments = input.segments();

        while let Some(segment) = segments.next() {
            if remain.len() <= segment.len() {
                if segment.starts_with(remain) {
                    remain = &[];
                    break;
                }
                drop(segments);
                return Fail((), input.into());
            }

            let Some(r) = remain.strip_prefix(segment) else {
                drop(segments);
                return Fail((), input.into());
            };

            remain = r;
        }

        drop(segments);

        if !remain.is_empty() {
            return Fail((), input.into());
        }

        Done(self.items, input.advance(self.items.len()))
    }
}

pub struct Single<'a, T>
where
    T: PartialEq,
{
    item: &'a T,
}

impl<'a, T, S> Parser<S> for Single<'a, T>
where
    T: PartialEq,
    S: ParcomStream<Segment = [T]>,
{
    type Output = &'a T;
    type Error = ();
    type Fault = Never;

    fn parse(&self, input: S) -> ParserResult<S, Self> {
        let head = input.segments().flatten().next();
        match head {
            Some(c) if c == self.item => Done(self.item, input.advance(1)),
            _ => Fail((), input.into()),
        }
    }
}
