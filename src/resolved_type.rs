use simple_error::SimpleResult;
use std::ops::Range;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, ResolveOffset};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ResolvedType {
    pub actual_range: Range<u64>,
    pub aligned_range: Range<u64>,
    pub field_name: Option<String>,
    pub field_type: H2Type,
}

impl ResolvedType {
    // This is a simpler way to display the type for the right part of the
    // context
    pub fn to_string(&self, offset: ResolveOffset) -> SimpleResult<String> {
        self.field_type.to_string(offset.at(self.actual_range.start))
    }
}
