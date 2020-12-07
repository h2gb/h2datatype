use simple_error::{bail, SimpleResult};

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2Types, H2TypeTrait, Offset};
use crate::alignment::Alignment;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: u64,
}

impl H2Array {
    // TODO: We need to prevent zero-length arrays
    pub fn new_aligned(alignment: Alignment, length: u64, field_type: H2Type) -> SimpleResult<H2Type> {
        if length == 0 {
            bail!("Arrays must be at least one element long");
        }

        Ok(H2Type::new(alignment, H2Types::H2Array(Self {
            field_type: Box::new(field_type),
            length: length,
        })))
    }

    pub fn new(length: u64, field_type: H2Type) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, length, field_type)
    }
}

impl H2TypeTrait for H2Array {
    fn is_static(&self) -> bool {
        // Offload the is_static() question to the child field type
        self.field_type.is_static()
    }

    //fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
    //}

    fn children(&self, _offset: Offset) -> SimpleResult<Vec<H2Type>> {
        // Just clone the child type over and over
        Ok((0..self.length).into_iter().map(|_index| {
            self.field_type.as_ref().clone()
        }).collect())
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        // Because the collect() expects a result, this will end and bubble
        // up errors automatically!
        let strings: Vec<String> = self.children_with_range(offset)?.iter().map(|(range, child)| {
            child.to_string(offset.at(range.start))
        }).collect::<SimpleResult<Vec<String>>>()?;

        Ok(format!("[{}]", strings.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    use crate::basic_type::{ASCII, StrictASCII};

    #[test]
    fn test_array_type() -> SimpleResult<()> {
        let data = b"ABCD".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Check the basics
        let a = H2Array::new(4, ASCII::new(StrictASCII::Permissive))?;
        assert_eq!(true, a.is_static());
        assert_eq!(4, a.actual_size(offset)?);
        assert_eq!(4, a.aligned_size(offset)?);
        assert_eq!(0..4, a.actual_range(offset)?);
        assert_eq!(0..4, a.aligned_range(offset)?);
        assert_eq!("[A, B, C, D]", a.to_string(offset)?);
        assert_eq!(0, a.related(offset)?.len());
        assert_eq!(4, a.children(offset)?.len());

        // Check the resolved version
        let r = a.resolve(offset)?;
        assert_eq!(4, r.actual_size());
        assert_eq!(4, r.aligned_size());
        assert_eq!(0..4, r.actual_range);
        assert_eq!(0..4, r.aligned_range);
        assert_eq!("[A, B, C, D]", r.value);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(0..1, r.children[0].aligned_range);
        assert_eq!(1..2, r.children[1].aligned_range);
        assert_eq!(2..3, r.children[2].aligned_range);
        assert_eq!(3..4, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("A", r.children[0].value);
        assert_eq!("B", r.children[1].value);
        assert_eq!("C", r.children[2].value);
        assert_eq!("D", r.children[3].value);

        Ok(())
    }

    #[test]
    fn test_array_type_aligned() -> SimpleResult<()> {
        let data = b"ABCD".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Check the basics
        let a = H2Array::new_aligned(Alignment::Loose(8), 4, ASCII::new(StrictASCII::Permissive))?;
        assert_eq!(true, a.is_static());
        assert_eq!(4, a.actual_size(offset)?);
        assert_eq!(8, a.aligned_size(offset)?);
        assert_eq!(0..4, a.actual_range(offset)?);
        assert_eq!(0..8, a.aligned_range(offset)?);
        assert_eq!("[A, B, C, D]", a.to_string(offset)?);
        assert_eq!(0, a.related(offset)?.len());
        assert_eq!(4, a.children(offset)?.len());

        // Check the resolved version
        let r = a.resolve(offset)?;
        assert_eq!(4, r.actual_size());
        assert_eq!(8, r.aligned_size());
        assert_eq!(0..4, r.actual_range);
        assert_eq!(0..8, r.aligned_range);
        assert_eq!("[A, B, C, D]", r.value);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(0..1, r.children[0].aligned_range);
        assert_eq!(1..2, r.children[1].aligned_range);
        assert_eq!(2..3, r.children[2].aligned_range);
        assert_eq!(3..4, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("A", r.children[0].value);
        assert_eq!("B", r.children[1].value);
        assert_eq!("C", r.children[2].value);
        assert_eq!("D", r.children[3].value);

        Ok(())
    }

    #[test]
    fn test_array_type_aligned_elements() -> SimpleResult<()> {
        let data = b"AxxxBxxxCxxxDxxx".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Check the basics
        let a = H2Array::new(4, ASCII::new_aligned(Alignment::Loose(4), StrictASCII::Permissive))?;
        assert_eq!(true, a.is_static());
        assert_eq!(16,  a.actual_size(offset)?);
        assert_eq!(16, a.aligned_size(offset)?);
        assert_eq!(0..16,  a.actual_range(offset)?);
        assert_eq!(0..16, a.aligned_range(offset)?);
        assert_eq!("[A, B, C, D]", a.to_string(offset)?);
        assert_eq!(0, a.related(offset)?.len());
        assert_eq!(4, a.children(offset)?.len());

        // Check the resolved version
        let r = a.resolve(offset)?;
        assert_eq!(16, r.actual_size());
        assert_eq!(16, r.aligned_size());
        assert_eq!(0..16, r.actual_range);
        assert_eq!(0..16, r.aligned_range);
        assert_eq!("[A, B, C, D]", r.value);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(0..1,   r.children[0].actual_range);
        assert_eq!(4..5,   r.children[1].actual_range);
        assert_eq!(8..9,   r.children[2].actual_range);
        assert_eq!(12..13, r.children[3].actual_range);

        // Make sure the aligned range is right
        assert_eq!(0..4,   r.children[0].aligned_range);
        assert_eq!(4..8,   r.children[1].aligned_range);
        assert_eq!(8..12,  r.children[2].aligned_range);
        assert_eq!(12..16, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("A", r.children[0].value);
        assert_eq!("B", r.children[1].value);
        assert_eq!("C", r.children[2].value);
        assert_eq!("D", r.children[3].value);

        Ok(())
    }

    #[test]
    fn test_array_type_aligned_and_aligned_elements() -> SimpleResult<()> {
        let data = b"AxxxBxxxCxxxDxxx".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Check the basics (align to 5, which is awkward but easy to check)
        let a = H2Array::new_aligned(Alignment::Loose(5), 4, ASCII::new_aligned(Alignment::Loose(4), StrictASCII::Permissive))?;
        assert_eq!(true, a.is_static());
        assert_eq!(16,  a.actual_size(offset)?);
        assert_eq!(20, a.aligned_size(offset)?);
        assert_eq!(0..16,  a.actual_range(offset)?);
        assert_eq!(0..20, a.aligned_range(offset)?);
        assert_eq!("[A, B, C, D]", a.to_string(offset)?);
        assert_eq!(0, a.related(offset)?.len());
        assert_eq!(4, a.children(offset)?.len());

        // Check the resolved version
        let r = a.resolve(offset)?;
        assert_eq!(16, r.actual_size());
        assert_eq!(20, r.aligned_size());
        assert_eq!(0..16, r.actual_range);
        assert_eq!(0..20, r.aligned_range);
        assert_eq!("[A, B, C, D]", r.value);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(0..1,   r.children[0].actual_range);
        assert_eq!(4..5,   r.children[1].actual_range);
        assert_eq!(8..9,   r.children[2].actual_range);
        assert_eq!(12..13, r.children[3].actual_range);

        // Make sure the aligned range is right
        assert_eq!(0..4,   r.children[0].aligned_range);
        assert_eq!(4..8,   r.children[1].aligned_range);
        assert_eq!(8..12,  r.children[2].aligned_range);
        assert_eq!(12..16, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("A", r.children[0].value);
        assert_eq!("B", r.children[1].value);
        assert_eq!("C", r.children[2].value);
        assert_eq!("D", r.children[3].value);

        Ok(())
    }

    #[test]
    fn test_array_type_aligned_and_offset_elements() -> SimpleResult<()> {
        let data = b"xAxxxBxxxCxxxDxx".to_vec();
        let offset = Offset::Dynamic(Context::new(&data).at(1));

        let a = H2Array::new(4, ASCII::new_aligned(Alignment::Loose(4), StrictASCII::Permissive))?;
        assert_eq!(true, a.is_static());
        assert_eq!(16,  a.actual_size(offset)?);
        assert_eq!(16, a.aligned_size(offset)?);
        assert_eq!(1..17,  a.actual_range(offset)?);
        assert_eq!(1..17, a.aligned_range(offset)?);
        assert_eq!("[A, B, C, D]", a.to_string(offset)?);
        assert_eq!(0, a.related(offset)?.len());
        assert_eq!(4, a.children(offset)?.len());

        // Check the resolved version
        let r = a.resolve(offset)?;
        assert_eq!(16, r.actual_size());
        assert_eq!(16, r.aligned_size());
        assert_eq!(1..17, r.actual_range);
        assert_eq!(1..17, r.aligned_range);
        assert_eq!("[A, B, C, D]", r.value);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(1..2,   r.children[0].actual_range);
        assert_eq!(5..6,   r.children[1].actual_range);
        assert_eq!(9..10,  r.children[2].actual_range);
        assert_eq!(13..14, r.children[3].actual_range);

        // Make sure the aligned range is right
        assert_eq!(1..5,   r.children[0].aligned_range);
        assert_eq!(5..9,   r.children[1].aligned_range);
        assert_eq!(9..13,  r.children[2].aligned_range);
        assert_eq!(13..17, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("A", r.children[0].value);
        assert_eq!("B", r.children[1].value);
        assert_eq!("C", r.children[2].value);
        assert_eq!("D", r.children[3].value);

        Ok(())
    }

//     #[test]
//     fn test_array() -> SimpleResult<()> {
//         let data = b"AAAABBBBCCCCDDDD".to_vec();
//         let s_offset = Offset::Static(0);
//         let d_offset = Offset::Dynamic(Context::new(&data));

//         // An array of 4 32-bit unsigned integers
//         let t = H2Array::new(4,
//             H2Number::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default()))
//         );

//         assert_eq!(true, t.is_static());
//         assert_eq!(16, t.actual_size(s_offset)?);

//         assert_eq!(4, t.resolve_partial(s_offset)?.len());
//         assert_eq!(4, t.resolve_partial(d_offset)?.len());

//         let resolved = t.resolve_full(d_offset)?;
//         assert_eq!(4, resolved.len());

//         assert_eq!(0..4, resolved[0].actual_range);
//         assert_eq!("0x41414141", resolved[0].to_string());

//         assert_eq!(4..8, resolved[1].actual_range);
//         assert_eq!("0x42424242", resolved[1].to_string());

//         assert_eq!(8..12, resolved[2].actual_range);
//         assert_eq!("0x43434343", resolved[2].to_string());

//         assert_eq!(12..16, resolved[3].actual_range);
//         assert_eq!("0x44444444", resolved[3].to_string());

//         Ok(())
//     }

//     #[test]
//     fn test_nested_array() -> SimpleResult<()> {
//         let data = b"\x00\x00\x00\x00\x7f\x7f\x7f\x7f\x80\x80\xff\xff".to_vec();
//         let s_offset = Offset::Static(0);
//         let d_offset = Offset::Dynamic(Context::new(&data));

//         // An array of 4 4-element I8 arrays that will print as decimal
//         let t = H2Array::new(4,
//             H2Array::new(3,
//                 H2Number::new(SizedDefinition::I8, SizedDisplay::Decimal)
//             ),
//         );

//         assert_eq!(12, t.actual_size(s_offset)?);
//         assert_eq!(12, t.actual_size(d_offset)?);

//         // Should have 4 direct children
//         assert_eq!(4, t.resolve_partial(s_offset)?.len());
//         assert_eq!(4, t.resolve_partial(d_offset)?.len());

//         // And a total length of 12
//         let resolved = t.resolve_full(d_offset)?;
//         assert_eq!(12, resolved.len());

//         assert_eq!("0",    resolved[0].to_string());
//         assert_eq!("0",    resolved[1].to_string());
//         assert_eq!("0",    resolved[2].to_string());
//         assert_eq!("0",    resolved[3].to_string());

//         assert_eq!("127",  resolved[4].to_string());
//         assert_eq!("127",  resolved[5].to_string());
//         assert_eq!("127",  resolved[6].to_string());
//         assert_eq!("127",  resolved[7].to_string());

//         assert_eq!("-128", resolved[8].to_string());
//         assert_eq!("-128", resolved[9].to_string());
//         assert_eq!("-1",  resolved[10].to_string());
//         assert_eq!("-1",  resolved[11].to_string());

//         Ok(())
//     }

//     #[test]
//     fn test_alignment() -> SimpleResult<()> {
//         let data = b"AAAABBBBCCCCDDDD".to_vec();
//         let s_offset = Offset::Static(0);
//         let d_offset = Offset::Dynamic(Context::new(&data));

//         // An array of 4 32-bit unsigned integers
//         let t = H2Array::new(4,
//             H2Number::new_aligned(Alignment::Loose(4), SizedDefinition::U8, SizedDisplay::Hex(Default::default()))
//         );

//         // Even though it's 4x U8 values, with padding it should be 16
//         // (We don't want the array itself to be aligned - hence, `Align::No`)
//         assert_eq!(16, t.actual_size(s_offset)?);
//         assert_eq!(16, t.actual_size(d_offset)?);

//         let children = t.resolve_partial(d_offset)?;
//         assert_eq!(4, children.len());
//         assert_eq!(0..1, children[0].actual_range);
//         assert_eq!("0x41", children[0].to_string());

//         let resolved = t.resolve_full(d_offset)?;
//         assert_eq!(4, resolved.len());

//         assert_eq!(0..1,   resolved[0].actual_range);
//         assert_eq!("0x41", resolved[0].to_string());

//         assert_eq!(4..5,   resolved[1].actual_range);
//         assert_eq!("0x42", resolved[1].to_string());

//         assert_eq!(8..9,   resolved[2].actual_range);
//         assert_eq!("0x43", resolved[2].to_string());

//         assert_eq!(12..13, resolved[3].actual_range);
//         assert_eq!("0x44", resolved[3].to_string());

//         Ok(())
//     }

//     #[test]
//     fn test_nested_alignment() -> SimpleResult<()> {
//         let data = b"AABBCCDDEEFFGGHH".to_vec();
//         let s_offset = Offset::Static(0);
//         let d_offset = Offset::Dynamic(Context::new(&data));

//         // An array of 4 elements
//         let t = H2Array::new(4,
//             // Array of 2 elements, each of which is aligned to a 4-byte boundary
//             H2Array::new_aligned(Alignment::Loose(4), 2,
//                 // Each element is a 1-byte hex number aligned to a 2-byte bounary
//                 H2Number::new_aligned(Alignment::Loose(2), SizedDefinition::U8, SizedDisplay::Hex(Default::default()))
//             )
//         );

//         // Even though it's 4x U8 values, with padding it should be 16
//         assert_eq!(16, t.actual_size(s_offset)?);
//         assert_eq!(16, t.actual_size(d_offset)?);

//         // Make sure there are 4 direct children
//         assert_eq!(4, t.resolve_partial(d_offset)?.len());

//         // Make sure there are 8 total values
//         let resolved = t.resolve_full(d_offset)?;
//         assert_eq!(8, resolved.len());

//         assert_eq!(0..1,   resolved[0].actual_range);
//         assert_eq!("0x41", resolved[0].to_string());

//         assert_eq!(2..3,   resolved[1].actual_range);
//         assert_eq!("0x42", resolved[1].to_string());

//         assert_eq!(4..5,   resolved[2].actual_range);
//         assert_eq!("0x43", resolved[2].to_string());

//         assert_eq!(6..7,   resolved[3].actual_range);
//         assert_eq!("0x44", resolved[3].to_string());

//         Ok(())
//     }

//     #[test]
//     fn test_offset_padding() -> SimpleResult<()> {
//         // The - characters will be in the padding
//         let data = b"-A---B---C---D--".to_vec();
//         let d_offset = Offset::Dynamic(Context::new(&data).at(1));

//         // An array of 4 32-bit unsigned integers
//         let t = H2Array::new(4,
//             ASCII::new_aligned(Alignment::Loose(4))
//         );

//         assert_eq!(15, t.actual_size(d_offset)?);

//         // Resolve starting at 1, but due to the padding the range will be
//         // 0..4
//         let resolved = t.resolve_full(d_offset)?;
//         assert_eq!(4, resolved.len());

//         assert_eq!(1..2, resolved[0].actual_range);
//         assert_eq!(1..4, resolved[0].aligned_range);
//         assert_eq!("A", resolved[0].to_string());

//         assert_eq!(5..6, resolved[1].actual_range);
//         assert_eq!(5..8, resolved[1].aligned_range);
//         assert_eq!("B", resolved[1].to_string());

//         assert_eq!(9..10, resolved[2].actual_range);
//         assert_eq!(9..12, resolved[2].aligned_range);
//         assert_eq!("C", resolved[2].to_string());

//         assert_eq!(13..14, resolved[3].actual_range);
//         assert_eq!(13..16, resolved[3].aligned_range);
//         assert_eq!("D", resolved[3].to_string());

//         Ok(())
//     }

    #[test]
    fn test_dynamic_array() -> SimpleResult<()> {
        Ok(())
    }
}
