use crate::Location;

pub trait ParseStream<L: Location<Self::Segment>>: LocatableStream<L> + RewindStream {}

impl<L: Location<Self::Segment>, S: LocatableStream<L> + RewindStream> ParseStream<L> for S {}

pub trait LocatableStream<L>: Stream
where
    L: Location<<Self as Stream>::Segment>,
{
    fn location(&self, nth: usize) -> L;
}

pub trait IntoLocatable: Sized + Stream {
    type Locatable<L>: LocatableStream<L, Segment = Self::Segment>
    where
        L: Location<Self::Segment>;

    fn into_locatable_at<L>(self, location: L) -> Self::Locatable<L>
    where
        L: Location<Self::Segment>;

    fn into_locatable<L>(self) -> Self::Locatable<L>
    where
        L: Location<Self::Segment> + std::default::Default,
    {
        self.into_locatable_at(L::default())
    }
}

pub trait RewindStream: Stream {
    type Anchor;

    fn anchor(&self) -> Self::Anchor;
    fn rewind(self, anchor: Self::Anchor) -> Self;
}

pub trait Stream {
    type Segment: ?Sized;
    type Iter<'a>: 'a + Iterator<Item = &'a Self::Segment>
    where
        Self: 'a;

    fn segments(&self) -> Self::Iter<'_>;
    fn advance(self, count: usize) -> Self;
}

pub trait BindableStream<L>: LocatableStream<L>
where
    L: Location<Self::Segment>,
{
    fn bind<T>(self, location: L, item: T) -> Self;
    fn get<T>(&self, location: L) -> Option<&T>;
}
