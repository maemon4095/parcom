#[cfg(feature = "foreign")]
pub mod foreign;
#[cfg(feature = "packrat")]
pub mod packrat;
#[cfg(feature = "standard")]
pub mod standard;

pub use parcom_core::*;

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
