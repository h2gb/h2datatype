#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::basic_type::h2number::H2Number;
use crate::basic_type::h2pointer::H2Pointer;
use crate::basic_type::character::Character;
use crate::basic_type::ipv4::IPv4;
use crate::basic_type::ipv6::IPv6;
use crate::basic_type::unicode::Unicode;
use crate::complex_type::h2array::H2Array;
use crate::complex_type::h2struct::H2Struct;
// use crate::dynamic_type::ntstring::NTString;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum H2Types {
    // Basic
    H2Number(H2Number),
    H2Pointer(H2Pointer),
    Character(Character),
    IPv4(IPv4),
    IPv6(IPv6),
    Unicode(Unicode),

    // Complex
    H2Array(H2Array),
    H2Struct(H2Struct),

    // Dynamic
    // NTString(dynamic_type::ntstring::NTString),
}
