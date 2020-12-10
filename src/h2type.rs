use simple_error::SimpleResult;
use std::ops::Range;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{Alignment, H2Types, H2TypeTrait, Offset, ResolvedType};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct H2Type {
    pub field: H2Types,
    pub alignment: Alignment,
}

impl H2Type {
    pub fn new(alignment: Alignment, field: H2Types) -> Self {
        Self {
            field: field,
            alignment: alignment,
        }
    }

    fn field_type(&self) -> &dyn H2TypeTrait {
        match &self.field {
            // Basic
            H2Types::H2Number(t)  => t,
            H2Types::H2Pointer(t) => t,
            H2Types::Character(t) => t,

            H2Types::IPv4(t)      => t,
            H2Types::IPv6(t)      => t,

            // Complex
            H2Types::H2Array(t)   => t,
            H2Types::H2Enum(t)    => t,
            H2Types::H2Struct(t)  => t,

            // Strings
            H2Types::LString(t)   => t,
            H2Types::NTString(t)  => t,
            H2Types::LPString(t)  => t,
        }
    }

    // Is the size known ahead of time?
    pub fn is_static(&self) -> bool {
        self.field_type().is_static()
    }

    /// Size of just the field - no padding
    pub fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        self.field_type().actual_size(offset)
    }

    /// Size including padding either before or after
    pub fn aligned_size(&self, offset: Offset) -> SimpleResult<u64> {
        self.field_type().aligned_size(offset, self.alignment)
    }

    pub fn actual_range(&self, offset: Offset) -> SimpleResult<Range<u64>> {
        self.field_type().range(offset, Alignment::None)
    }

    // What range does this cover? Use [`Alignment::None`] to get the size
    // without alignment
    //
    // As long as actual_size() works, this will automatically work.
    pub fn aligned_range(&self, offset: Offset) -> SimpleResult<Range<u64>> {
        self.field_type().range(offset, self.alignment)
    }

    // Get "related" nodes - ie, what a pointer points to
    pub fn related(&self, offset: Offset) -> SimpleResult<Vec<(u64, H2Type)>> {
        self.field_type().related(offset)
    }

    pub fn children(&self, offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        self.field_type().children(offset)
    }

    pub fn resolve(&self, offset: Offset, name: Option<String>) -> SimpleResult<ResolvedType> {
        self.field_type().resolve(offset, self.alignment, name)
    }

    pub fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        self.field_type().to_string(offset)
    }

    pub fn can_be_char(&self) -> bool {
        self.field_type().can_be_char()
    }

    pub fn to_char(&self, offset: Offset) -> SimpleResult<char> {
        self.field_type().to_char(offset)
    }

    pub fn can_be_u64(&self) -> bool {
        self.field_type().can_be_u64()
    }

    pub fn to_u64(&self, offset: Offset) -> SimpleResult<u64> {
        self.field_type().to_u64(offset)
    }

    pub fn to_i64(&self, offset: Offset) -> SimpleResult<i64> {
        self.field_type().to_i64(offset)
    }

    pub fn can_be_i64(&self) -> bool {
        self.field_type().can_be_i64()
    }
}
