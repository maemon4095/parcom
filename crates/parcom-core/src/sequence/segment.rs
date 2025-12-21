pub trait SequenceSegment {
    type Length: Default + std::cmp::Ord;

    fn len(&self) -> Self::Length;
    fn split_at(&self, mid: Self::Length) -> (&Self, &Self);
}
