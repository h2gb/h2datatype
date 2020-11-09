use simple_error::SimpleResult;

use crate::{ResolveOffset, ResolvedType, H2Type};

pub trait H2TypeTrait {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool;

    // Get the static size, if possible
    fn size(&self, offset: ResolveOffset) -> SimpleResult<u64>;

    // Get "child" nodes (array elements, struct body, etc), if possible
    // Empty vector = a leaf node
    fn resolve_partial(&self, _offset: ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        Ok(vec![])
    }

    // Get the user-facing name of the type
    fn to_string(&self, offset: ResolveOffset) -> SimpleResult<String>;

    // Get "related" nodes - ie, what a pointer points to
    fn related(&self, _offset: ResolveOffset) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }
}

