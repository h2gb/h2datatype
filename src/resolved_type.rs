use std::ops::Range;
use std::fmt;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::H2Type;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ResolvedType {
    pub actual_range: Range<u64>,
    pub aligned_range: Range<u64>,
    pub field_type: H2Type,

    pub field_name: Option<String>,
    pub value: String,
}

impl fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
