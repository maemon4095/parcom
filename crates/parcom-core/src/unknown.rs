use crate::RewindStream;

#[derive(Debug, Clone)]
pub struct UnknownLocation<S>(S);

impl<S> UnknownLocation<S> {
    pub unsafe fn unwrap(self) -> S {
        self.0
    }
    pub fn as_ref(&self) -> UnknownLocation<&S> {
        UnknownLocation(&self.0)
    }
}

impl<S: RewindStream> UnknownLocation<S> {
    pub fn rewind(self, anchor: S::Anchor) -> S::Rewind {
        self.0.rewind(anchor)
    }
}

impl<S> From<S> for UnknownLocation<S> {
    fn from(value: S) -> Self {
        Self(value)
    }
}
