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

    pub field_name: Option<String>,
    pub value: String,

    pub children: Vec<ResolvedType>,
    pub related: Vec<(u64, H2Type)>,
}

impl fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.field_name {
            Some(n) => write!(f, "{}: {}", n, self.value),
            None    => write!(f, "{}", self.value),
        }
    }
}

impl ResolvedType {
    pub fn actual_size(&self) -> u64 {
        self.actual_range.end - self.actual_range.start
    }

    pub fn aligned_size(&self) -> u64 {
        self.aligned_range.end - self.aligned_range.start
    }
}
