#[cfg(feature = "foreign")]
pub mod foreign;
#[cfg(feature = "packrat")]
pub mod packrat;
#[cfg(feature = "standard")]
pub mod standard;

pub use parcom_core::*;
pub mod prelude {
    pub use parcom_core::*;
    pub use ParseResult::*;
}

mod internal {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}
