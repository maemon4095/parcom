#[cfg(feature = "foreign")]
pub mod foreign;
#[cfg(feature = "packrat")]
pub mod packrat;
#[cfg(feature = "standard")]
pub mod standard;

pub use parcom_core::*;
pub type ParserResult<S, P> =
    ParseResult<S, <P as Parser<S>>::Output, <P as Parser<S>>::Error, <P as Parser<S>>::Fault>;

pub mod prelude {
    pub use crate::ParserResult;
    pub use parcom_core::ParseResult::*;
    pub use parcom_core::Result::*;
    pub use parcom_core::*;
}

mod internal {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}
