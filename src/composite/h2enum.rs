#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use simple_error::{bail, SimpleResult};
use std::cmp;
use std::ops::Range;

use crate::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines an enum - a selection of values in the same memory, of which one
/// is used.
///
/// Any number of different types can be defined, and the length of the field
/// will be the length of the longest one. When resolved, the results will have
/// the same starting address.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct H2Enum {
    /// An array of strings and types (which might be other types)
    variants: Vec<(String, H2Type)>,
}

impl H2Enum {
    pub fn new_aligned(alignment: Alignment, variants: Vec<(String, H2Type)>) -> SimpleResult<H2Type> {
        if variants.len() == 0 {
            bail!("Enums must have at least one variant");
        }

        Ok(H2Type::new(alignment, H2Types::H2Enum(Self {
            variants: variants,
        })))
    }

    pub fn new(variants: Vec<(String, H2Type)>) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, variants)
    }
}

impl H2TypeTrait for H2Enum {
    fn is_static(&self) -> bool {
        // Loop over each field - return an object as soon as is_static() is
        // false
        self.variants.iter().find(|(_, t)| {
            t.is_static() == false
        }).is_none()
    }

    /// We must implement this, because unlike others the end isn't necessarily
    /// the end of the last child
    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        // Check each variant's length, saving the longest
        self.variants.iter().try_fold(0, |sum, (_, t)| {
            // This returns the bigger of the current value or the new value
            Ok(cmp::max(t.aligned_size(offset)?, sum))
        })
    }

    fn children(&self, _offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        Ok(self.variants.iter().map(|(name, field_type)| {
            (Some(name.clone()), field_type.clone())
        }).collect())
    }

    /// We must implement this ourselves, because all children will start at the
    /// same offset (instead of being sequential)
    fn children_with_range(&self, offset: Offset) -> SimpleResult<Vec<(Range<u64>, Option<String>, H2Type)>> {
        self.variants.iter().map(|(name, field_type)| {
            Ok((field_type.aligned_range(offset)?, Some(name.clone()), field_type.clone()))
        }).collect::<SimpleResult<Vec<_>>>()
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        let strings: Vec<String> = self.children_with_range(offset)?.into_iter().map(|(range, name, child)| {
            Ok(format!("{}: {}", name.unwrap_or("<name unknown>".to_string()), child.to_display(offset.at(range.start))?))
        }).collect::<SimpleResult<Vec<String>>>()?;

        Ok(format!("{{ {} }}", strings.join(" | ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};

    use crate::simple::H2Number;
    use crate::simple::character::{ASCII, StrictASCII};
    use crate::composite::H2Array;

    #[test]
    fn test_enum() -> SimpleResult<()> {
        let data = b"xxxABCDEFGHIJKLMNOP".to_vec();
        let offset = Offset::Dynamic(Context::new_at(&data, 3));

        let e = H2Enum::new_aligned(Alignment::Loose(16), vec![
            (
                "u16".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(3),
                    SizedDefinition::U16(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "u32".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "array".to_string(),
                H2Array::new_aligned(
                    Alignment::Loose(12),
                    8,
                    ASCII::new(StrictASCII::Permissive),
                )?,
            ),
            (
                "u8octal".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(4),
                    SizedDefinition::U8,
                    SizedDisplay::Octal(Default::default()),
                )
            ),
        ])?;

        // Check the basics
        assert_eq!(true, e.is_static());
        assert_eq!(12, e.actual_size(offset)?);
        assert_eq!(16, e.aligned_size(offset)?);
        assert_eq!(3..15, e.actual_range(offset)?);
        assert_eq!(3..19, e.aligned_range(offset)?);
        assert_eq!("{ u16: 0x4142 | u32: 0x44434241 | array: [ 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H' ] | u8octal: 0o101 }", e.to_display(offset)?);
        assert_eq!(0, e.related(offset)?.len());
        assert_eq!(4, e.children(offset)?.len());

        // Check the resolved version
        let r = e.resolve(offset, None)?;
        assert_eq!(12, r.actual_size());
        assert_eq!(16, r.aligned_size());
        assert_eq!(3..15, r.actual_range);
        assert_eq!(3..19, r.aligned_range);
        assert_eq!("{ u16: 0x4142 | u32: 0x44434241 | array: [ 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H' ] | u8octal: 0o101 }", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(3..5,  r.children[0].actual_range);
        assert_eq!(3..7,  r.children[1].actual_range);
        assert_eq!(3..11, r.children[2].actual_range);
        assert_eq!(3..4,  r.children[3].actual_range);

        assert_eq!(3..6,  r.children[0].aligned_range);
        assert_eq!(3..7,  r.children[1].aligned_range);
        assert_eq!(3..15, r.children[2].aligned_range);
        assert_eq!(3..7,  r.children[3].aligned_range);

        Ok(())
    }
}
