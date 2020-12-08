#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::basic_type::*;
use crate::complex_type::*;
// use crate::dynamic_type::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum H2Types {
    // Basic
    H2Number(H2Number),
    H2Pointer(H2Pointer),

    IPv4(IPv4),
    IPv6(IPv6),

    ASCII(ASCII),
    UTF8(UTF8),
    UTF16(UTF16),
    UTF32(UTF32),

    // Complex
    H2Array(H2Array),
    H2Enum(H2Enum),
    H2Struct(H2Struct),

    // Dynamic
    // NTString(dynamic_type::ntstring::NTString),
}
