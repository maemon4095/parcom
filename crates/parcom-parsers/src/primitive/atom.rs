use parcom_base::error::Miss;
use parcom_core::{
    ParseResult, Parser, ParserOnce, ParserResult, SegmentIterator, Stream, StreamSegment,
};

pub fn atom<P: AtomPattern>(pattern: P) -> Atom<P> {
    Atom::new(pattern)
}

pub struct Atom<P: AtomPattern> {
    pattern: P,
}

impl<P: AtomPattern> Atom<P> {
    pub fn new(pattern: P) -> Self {
        Self { pattern }
    }
}

impl<P: AtomPattern, S: Stream<Segment = P::Segment>> ParserOnce<S> for Atom<P> {
    type Output = ();
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<P: AtomPattern, S: Stream<Segment = P::Segment>> Parser<S> for Atom<P> {
    async fn parse(&self, mut input: S) -> ParserResult<S, Self> {
        let mut remain = self.pattern.pattern();
        let mut segments = input.segments();

        while let Some(segment) = segments.next(remain.len()).await {
            let segment = match segment {
                Ok(v) => v,
                Err(e) => return ParseResult::StreamErr(e, input.into()),
            };
            if segment.len() >= remain.len() {
                let s = segment.split_at(remain.len()).0;
                return if s == remain {
                    ParseResult::Done((), input.advance(remain.len()).await)
                } else {
                    ParseResult::Fail(Miss(()), input.into())
                };
            }

            let (p, r) = remain.split_at(segment.len());

            if &*segment != p {
                return ParseResult::Fail(Miss(()), input.into());
            }

            remain = r;
        }

        return ParseResult::Fail(().into(), input.into());
    }
}

pub trait AtomPattern {
    type Segment: ?Sized + StreamSegment + PartialEq;

    fn pattern(&self) -> &Self::Segment;
}

impl<'a> AtomPattern for &'a str {
    type Segment = str;

    fn pattern(&self) -> &Self::Segment {
        self
    }
}

impl<'a, T: PartialEq> AtomPattern for &'a [T] {
    type Segment = [T];

    fn pattern(&self) -> &Self::Segment {
        self
    }
}

impl AtomPattern for String {
    type Segment = str;

    fn pattern(&self) -> &Self::Segment {
        self.as_str()
    }
}

impl<T: PartialEq> AtomPattern for Vec<T> {
    type Segment = [T];

    fn pattern(&self) -> &Self::Segment {
        self.as_slice()
    }
}

impl<T: PartialEq, const N: usize> AtomPattern for [T; N] {
    type Segment = [T];

    fn pattern(&self) -> &Self::Segment {
        self.as_slice()
    }
}
