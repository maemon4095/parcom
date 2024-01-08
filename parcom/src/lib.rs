#[cfg(feature = "foreign")]
pub mod foreign;

#[cfg(feature = "standard")]
pub mod standard;

#[cfg(feature = "locatable")]
pub mod locatable;

pub use parcom_core::*;

pub mod prelude {
    pub use crate::ParserResult;
    pub use parcom_core::ParseResult::*;
    pub use parcom_core::*;
}

mod internal {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}
