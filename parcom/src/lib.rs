#[cfg(feature = "std")]
pub use parcom_std as std;

pub use parcom_core::*;

pub mod prelude {
    pub use crate::ParserResult;
    pub use parcom_core::ParseResult::*;
    pub use parcom_core::*;
}
