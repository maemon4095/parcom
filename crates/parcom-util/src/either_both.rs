use parcom_core::{ParseError, ShouldNever, ShouldNeverExtension};

#[derive(Debug, Clone)]
pub enum EitherBoth<T0, T1> {
    First(T0),
    Last(T1),
    Both(T0, T1),
}

impl<T0, T1> EitherBoth<T0, T1> {
    pub fn always_first(self) -> T0
    where
        T1: ShouldNever,
    {
        match self {
            Self::First(e) => e,
            Self::Last(e) => e.never(),
            Self::Both(_, e) => e.never(),
        }
    }

    pub fn always_last(self) -> T1
    where
        T0: ShouldNever,
    {
        match self {
            Self::First(e) => e.never(),
            Self::Last(e) => e,
            Self::Both(e, _) => e.never(),
        }
    }
}

unsafe impl<T0: ShouldNever, T1: ShouldNever> ShouldNever for EitherBoth<T0, T1> {}

impl<T0: ParseError, T1: ParseError> ParseError for EitherBoth<T0, T1> {
    fn should_terminate(&self) -> bool {
        match self {
            EitherBoth::First(e) => e.should_terminate(),
            EitherBoth::Last(e) => e.should_terminate(),
            EitherBoth::Both(e0, e1) => e0.should_terminate() || e1.should_terminate(),
        }
    }
}
