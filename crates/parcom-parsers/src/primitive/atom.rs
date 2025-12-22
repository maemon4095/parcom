use parcom_core::{Parser, ParserOnce, ParserResult, SegmentStream, Sequence, SequenceSegment};
use parcom_util::{done, error::Miss, fail};

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

impl<P: AtomPattern, S: Sequence<Segment = P::Segment>> ParserOnce<S> for Atom<P> {
    type Output = ();
    type Error = Miss<()>;

    async fn parse_once(self, input: S) -> ParserResult<S, Self> {
        self.parse(input).await
    }
}

impl<P: AtomPattern, S: Sequence<Segment = P::Segment>> Parser<S> for Atom<P> {
    async fn parse(&self, mut input: S) -> ParserResult<S, Self> {
        let mut remain = self.pattern.pattern();
        let mut segments = input.segments();

        while let Some(segment) = segments.next().await {
            if segment.len() >= remain.len() {
                let s = segment.split_at(remain.len()).0;
                let matched = s == remain;
                drop(segments);
                return if matched {
                    done((), input.advance(remain.len()).await)
                } else {
                    fail(Miss(()), input)
                };
            }

            let (p, r) = remain.split_at(segment.len());

            if &*segment != p {
                drop(segments);
                return fail(Miss(()), input);
            }

            remain = r;
        }

        drop(segments);
        return fail((), input);
    }
}

pub trait AtomPattern {
    type Segment: ?Sized + SequenceSegment + PartialEq;

    fn pattern(&self) -> &Self::Segment;
}

impl<'a> AtomPattern for &'a str {
    type Segment = str;

    fn pattern(&self) -> &Self::Segment {
        self
    }
}

impl<'a, T: 'static + PartialEq> AtomPattern for &'a [T] {
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

impl<T: 'static + PartialEq> AtomPattern for Vec<T> {
    type Segment = [T];

    fn pattern(&self) -> &Self::Segment {
        self.as_slice()
    }
}

impl<T: 'static + PartialEq, const N: usize> AtomPattern for [T; N] {
    type Segment = [T];

    fn pattern(&self) -> &Self::Segment {
        self.as_slice()
    }
}
