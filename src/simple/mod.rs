//! Types that don't have subtypes.
//!
//! Keeping these types together in this module are a convention, there's no
//! firm rule.

mod h2number;
pub use h2number::*;

mod h2pointer;
pub use h2pointer::*;

mod ipv4;
pub use ipv4::*;

mod ipv6;
pub use ipv6::*;

pub mod character;