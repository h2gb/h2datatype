use simple_error::{bail, SimpleResult};
use std::ops::Range;

use crate::{Alignment, Offset, ResolvedType, H2Type};

pub trait H2TypeTrait {
    // Can all the elements be calculated ahead of time?
    fn is_static(&self) -> bool;

    // What's the size of the field (with no padding, though children will have padding here)
    //
    // We have a default implementation that resolves children, then takes the
    // end of the last and subtracts the start of the first. That can be really
    // slow, and only works with children, so implementations will frequently
    // want their own, I think.
    //
    // This also assumes that the last child has the last address. That doesn't
    // work for, say, H2Enum
    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        let children = self.children_with_range(offset)?;

        let first_range = match children.first() {
            Some((r, _, _)) => r,
            None => bail!("Can't calculate size with no child types"),
        };

        // This should never trigger, but just in case...
        let last_range = match children.last() {
            Some((r, _, _)) => r,
            None => bail!("Can't calculate size with no child types"),
        };

        Ok(last_range.end - first_range.start)
    }

    /// Size including padding either before or after
    fn aligned_size(&self, offset: Offset, alignment: Alignment) -> SimpleResult<u64> {
        let range = self.range(offset, alignment)?;

        Ok(range.end - range.start)
    }

    fn range(&self, offset: Offset, alignment: Alignment) -> SimpleResult<Range<u64>> {
        // Get the start and end
        let start = offset.position();
        let end   = start + self.actual_size(offset)?;

        // Do the rounding
        alignment.align(start..end)
    }

    // Get the user-facing value of this type.
    //
    // For static offsets, this should just be the field name or type.
    // For dynamic offsets, it should be the actual data, represented as text
    fn to_string(&self, offset: Offset) -> SimpleResult<String>;

    // Get "related" values - ie, what a pointer points to.
    //
    // Default: none
    fn related(&self, _offset: Offset) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }

    // Get the children as a vector of abstract H2Type values. This should be
    // implemented by any complex type that has subtypes.
    fn children(&self, _offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        Ok(vec![])
    }

    // Get a list of children types, and attach a range to each one based on its
    // size.
    //
    // There should be no need to override this.
    fn children_with_range(&self, offset: Offset) -> SimpleResult<Vec<(Range<u64>, Option<String>, H2Type)>> {
        let mut child_offset = offset;

        self.children(offset)?.into_iter().map(|(name, child)| {
            let range = child.aligned_range(child_offset)?;

            child_offset = offset.at(range.end);

            Ok((range, name, child.clone()))
            //// I'm not sure if this is a good idea, but...
            ////
            //// This ensures that the array field starts in the same "place"
            //// in every element of the array. I think that makes sense to my
            //// brain, but I'm not sure I like having this special case...
            //if let Alignment::Loose(a) = self.field_type.alignment {
            //    start = start + a;
            //} else {
            //    start = start + self.field_type.aligned_size(this_offset)?;
            //}
        }).collect::<SimpleResult<Vec<_>>>()
    }

    // Convert an abstract type to a concrete type. This is used recursively
    // to resolve children
    fn resolve(&self, offset: Offset, alignment: Alignment, field_name: Option<String>) -> SimpleResult<ResolvedType> {
        Ok(ResolvedType {
            actual_range: self.range(offset, Alignment::None)?,
            aligned_range: self.range(offset, alignment)?,

            field_name: field_name,
            value: self.to_string(offset)?,

            // Resolve the children here and now
            children: self.children_with_range(offset)?.into_iter().map(|(range, name, child)| {
                // Errors here will be handled by the collect
                child.resolve(offset.at(range.start), name)
            }).collect::<SimpleResult<Vec<ResolvedType>>>()?,

            related: self.related(offset)?,
        })
    }
}
