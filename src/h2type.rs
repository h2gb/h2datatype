use simple_error::SimpleResult;
use std::ops::Range;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{Alignment, H2Types, H2TypeTrait, ResolveOffset, ResolvedType};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct H2Type {
    field: H2Types,
    alignment: Alignment,
}

impl H2Type {
    pub fn new(alignment: Alignment, field: H2Types) -> Self {
        Self {
            field: field,
            alignment: alignment,
        }
    }

    pub fn field_type(&self) -> &dyn H2TypeTrait {
        match &self.field {
            // Basic
            H2Types::H2Number(t)  => t,
            H2Types::H2Pointer(t) => t,
            H2Types::Character(t) => t,
            H2Types::IPv4(t)      => t,
            H2Types::IPv6(t)      => t,
            H2Types::Unicode(t)   => t,

            // Complex
            H2Types::H2Array(t)   => t,
            H2Types::H2Struct(t)  => t,

            // Dynamic
            // H2Types::NTString(t)  => t,
        }
    }

    // Is the size known ahead of time?
    pub fn is_static(&self) -> bool {
        self.field_type().is_static()
    }

    /// Size of just the field - no padding
    pub fn actual_size(&self, offset: ResolveOffset) -> SimpleResult<u64> {
        self.field_type().size(offset)
    }

    /// Range of values this covers, with alignment padding built-in
    pub fn actual_range(&self, offset: ResolveOffset) -> SimpleResult<Range<u64>> {
        // Get the start and end
        let start = offset.position();
        let end   = offset.position() + self.actual_size(offset)?;

        // Do the rounding
        Ok(start..end)
    }

    /// Range of values this covers, with alignment padding built-in
    pub fn aligned_range(&self, offset: ResolveOffset) -> SimpleResult<Range<u64>> {
        // Get the start and end
        let start = offset.position();
        let end   = offset.position() + self.actual_size(offset)?;

        // Do the rounding
        self.alignment.align(start..end)
    }

    /// Size including padding either before or after
    pub fn aligned_size(&self, offset: ResolveOffset) -> SimpleResult<u64> {
        let range = self.aligned_range(offset)?;

        Ok(range.end - range.start)
    }

    pub fn resolve_partial(&self, offset: ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        self.field_type().resolve_partial(offset)
    }

    // Render as a string
    pub fn to_string(&self, offset: ResolveOffset) -> SimpleResult<String> {
        self.field_type().to_string(offset)
    }

    // Get "related" nodes - ie, what a pointer points to
    pub fn related(&self, offset: ResolveOffset) -> SimpleResult<Vec<(u64, H2Type)>> {
        self.field_type().related(offset)
    }

    pub fn resolve_full(&self, offset: ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        let children = self.resolve_partial(offset)?;
        let mut result: Vec<ResolvedType> = Vec::new();

        if children.len() == 0 {
            // No children? Return ourself!
            result.push(ResolvedType {
                actual_range: self.actual_range(offset)?,
                aligned_range: self.aligned_range(offset)?,
                field_name: None,
                field_type: self.clone(),
            });
        } else {
            // Children? Gotta get 'em all!
            for child in children.iter() {
                result.append(&mut child.field_type.resolve_full(offset.at(child.actual_range.start))?);
            }
        }

        Ok(result)
    }
}

