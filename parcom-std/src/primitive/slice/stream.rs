use parcom_core::{LocatableStream, IntoLocatable, Location, RewindStream, Stream};

#[derive(Debug)]
pub struct SliceStream<'me, T> {
    slice: &'me [T],
}

impl<'me, T> SliceStream<'me, T> {
    pub fn new(slice: &'me [T]) -> Self {
        Self { slice }
    }
}

impl<'me, T> Clone for SliceStream<'me, T> {
    fn clone(&self) -> Self {
        Self { slice: self.slice }
    }
}

impl<'me, T> Stream for SliceStream<'me, T> {
    type Segment = [T];

    type Iter<'a> = std::iter::Once<&'a [T]>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_> {
        std::iter::once(self.slice)
    }

    fn advance(mut self, count: usize) -> Self {
        self.slice = &self.slice[count..];
        self
    }
}

impl<'me, T> RewindStream for SliceStream<'me, T> {
    type Anchor = Anchor<'me, T>;

    fn anchor(&self) -> Self::Anchor {
        Anchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}

pub struct Anchor<'me, T> {
    stream: SliceStream<'me, T>,
}

impl<'me, T> IntoLocatable for SliceStream<'me, T> {
    type Locatable<L> = Locatable<'me, T, L> 
    where
        L: Location<Self::Segment>; 

    fn into_locatable_at<L>(self, location: L) -> Self::Locatable<L>
    where
        L: Location<Self::Segment> ,
    {
        Locatable {
            location,
            base: self,
        }
    }
}

#[derive(Debug)]
pub struct Locatable<'me, T, L>
where
    L: Location<[T]>,
{
    location: L,
    base: SliceStream<'me, T>,
}

impl<'me, T, L> Clone for Locatable<'me, T, L>
where
    L: Location<[T]>,
{
    fn clone(&self) -> Self {
        Self {
            location: self.location.clone(),
            base: self.base.clone(),
        }
    }
}

impl<'me, T, L> Stream for Locatable<'me, T, L>
where
    L: Location<[T]>,
{
    type Segment = <SliceStream<'me, T> as Stream>::Segment;

    type Iter<'a> = <SliceStream<'me, T> as Stream>::Iter<'a>
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

impl<'me, T, L> RewindStream for Locatable<'me, T, L>
where
    L: Location<[T]>,
{
    type Anchor = LocatableAnchor<'me, T, L>;

    fn anchor(&self) -> Self::Anchor {
        LocatableAnchor {
            stream: self.clone(),
        }
    }

    fn rewind(self, anchor: Self::Anchor) -> Self {
        anchor.stream
    }
}

pub struct LocatableAnchor<'me, T, L>
where
    L: Location<[T]>,
{
    stream: Locatable<'me, T, L>,
}

impl<'me, T, L> LocatableStream<L> for Locatable<'me, T, L>
where
    L: Location<[T]>,
{
    fn location(&self, nth: usize) -> L {
        self.location.clone().advance(&self.base.slice[..nth])
    }
}
