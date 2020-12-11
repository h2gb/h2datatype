#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use std::fmt;
use std::ops::Range;

use crate::H2Type;

/// The result of an [`H2Type`] being applied to a context.
///
/// Ultimately, a [`ResolvedType`] is "concrete" - that is, everything is
/// set in stone and unchanging. That means that getting information from it
/// is zero-cost, and requests can't fail (since they aren't working on
/// unexpected data).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ResolvedType {
    pub actual_range: Range<u64>,
    pub aligned_range: Range<u64>,

    pub field_name: Option<String>,
    pub value: String,

    pub children: Vec<ResolvedType>,
    pub related: Vec<(u64, H2Type)>,

    pub as_char: Option<char>,
    pub as_u64:  Option<u64>,
    pub as_i64:  Option<i64>,
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
