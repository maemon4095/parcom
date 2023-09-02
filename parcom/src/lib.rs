#[cfg(feature = "foreign")]
pub mod foreign;
#[cfg(feature = "packrat")]
pub mod packrat;
#[cfg(feature = "standard")]
pub mod standard;
#[cfg(feature = "stream")]
pub mod stream;


pub use parcom_core::*;

mod internal {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}
