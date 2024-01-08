use parcom_core::LocatableStream;

use crate::{Location, RewindStream,IntoLocatable, Stream};

#[derive(Debug, Clone)]
pub struct StrStream<'me> {
    str: &'me str,
}

impl<'me> StrStream<'me> {
    pub fn new(str: &'me str) -> Self {
        Self { str }
    }
}

impl<'me> Stream for StrStream<'me> {
    type Segment = str;

    type Iter<'a> = std::iter::Once<&'a str>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        std::iter::once(self.str)
    }

    fn advance(mut self, count: usize) -> Self {
        let mut chars = self.str.chars();
        for _ in 0..count {
            chars.next();
        }
        self.str = chars.as_str();
        self
    }
}
impl<'me> RewindStream for StrStream<'me> {
    type Anchor = Anchor<'me>;

    fn anchor(&self) -> Self::Anchor {
        Anchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}

pub struct Anchor<'me> {
    stream: StrStream<'me>,
}

impl<'me> IntoLocatable for StrStream<'me> {
    type Locatable<L>  = Locatable<'me, L>
    where
        L: Location<Self::Segment> ;

    fn into_locatable_at<L>(self, location: L) -> Self::Locatable<L>
    where
        L: Location<Self::Segment> {
        Locatable {
            location,
            base: self
        }
    }
}

#[derive(Debug, Clone)]
pub struct Locatable<'me, L: Location<str>> {
    location: L,
    base: StrStream<'me>,
}

impl<'me, L: Location<str>> Stream for Locatable<'me, L> {
    type Segment = str;

    type Iter<'a> = <StrStream<'me> as Stream>::Iter<'a> 
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        self.base.segments()
    }

    fn advance(mut self, count: usize) -> Self {
        self.location = self.location(count);
        self.base = self.base.advance(count);
        self
    }
}

impl<'me, L: Location<str>> RewindStream for Locatable<'me, L> {
    type Anchor = LocatableAnchor<'me, L>;

    fn anchor(&self) -> Self::Anchor {
        LocatableAnchor {
            stream: self.clone()
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}

pub struct LocatableAnchor<'me, L: Location<str>> {
    stream: Locatable<'me, L>
}

impl<'me, L: Location<str>> LocatableStream<L> for Locatable<'me, L> {
    fn location(&self, nth: usize) -> L {
        let str = self.base.str;
        let end = str.char_indices().nth(nth).map_or(str.len(), |e| e.0);
        self.location.clone().advance(&self.base.str[..end])
    }
}
