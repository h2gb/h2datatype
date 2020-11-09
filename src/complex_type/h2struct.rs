use simple_error::{bail, SimpleResult};

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2Types, ResolvedType, H2TypeTrait, ResolveOffset};
use crate::alignment::Alignment;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct H2Struct {
    fields: Vec<(String, H2Type)>,
}

impl H2Struct {
    // TODO: We need to prevent zero-length arrays
    pub fn new_aligned(alignment: Alignment, fields: Vec<(String, H2Type)>) -> H2Type {
        H2Type::new(alignment, H2Types::H2Struct(Self {
            fields: fields
        }))
    }

    pub fn new(fields: Vec<(String, H2Type)>) -> H2Type {
        Self::new_aligned(Alignment::None, fields)
    }
}

impl H2TypeTrait for H2Struct {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool {
        // Loop over each field
        self.fields.iter().find(|(_, t)|
            // Stop at the first non-static field
            t.is_static() == false
        ).is_some()
    }

    fn size(&self, offset: ResolveOffset) -> SimpleResult<u64> {
        let resolved = self.resolve_partial(offset)?;

        if let Some(first) = resolved.first() {
            if let Some(last) = resolved.last() {
                return Ok(last.aligned_range.end - first.aligned_range.start);
            } else {
                bail!("No elements");
            }
        } else {
            bail!("No elements");
        }
    }

    fn resolve_partial(&self, offset: ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        let mut start = offset.position();

        self.fields.iter().map(|(name, field_type)| {
            let this_offset = offset.at(start);

            let resolved = ResolvedType {
                actual_range: field_type.actual_range(this_offset)?,
                aligned_range: field_type.aligned_range(this_offset)?,
                field_name: Some(name.clone()),
                field_type: field_type.clone(),
            };

            start = resolved.aligned_range.end;

            Ok(resolved)
        }).collect::<SimpleResult<Vec<ResolvedType>>>()
    }

    // Get the user-facing name of the type
    fn to_string(&self, offset: ResolveOffset) -> SimpleResult<String> {
        let elements = self.resolve_partial(offset)?.iter().map(|t| {
            Ok(format!("{}: {}", t.field_name.clone().unwrap_or("(unnamed)".to_string()), t.to_string(offset)?))
        }).collect::<SimpleResult<Vec<String>>>()?;

        Ok(elements.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};

    use crate::basic_type::H2Number;
    use crate::complex_type::H2Array;

    #[test]
    fn test_struct() -> SimpleResult<()> {
        //           ----- hex ------ --hex-- -o- ----decimal----
        let data = b"\x00\x01\x02\x03\x00\x01\x0f\x0f\x0e\x0d\x0c".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        let t = H2Struct::new(vec![
            (
                "field_u32".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "field_u16".to_string(),
                H2Number::new(
                    SizedDefinition::U16(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "field_u8".to_string(),
                H2Number::new(
                    SizedDefinition::U8,
                    SizedDisplay::Octal(Default::default()),
                )
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Decimal,
                )
            ),
        ]);

        assert_eq!(11, t.actual_size(s_offset)?);
        assert_eq!(11, t.actual_size(d_offset)?);

        let resolved = t.resolve_full(d_offset)?;

        assert_eq!(4, resolved.len());
        assert_eq!(0..4, resolved[0].actual_range);
        assert_eq!("0x00010203", resolved[0].to_string(d_offset)?);

        assert_eq!(4..6, resolved[1].actual_range);
        assert_eq!("0x0001", resolved[1].to_string(d_offset)?);

        assert_eq!(6..7, resolved[2].actual_range);
        assert_eq!("0o17", resolved[2].to_string(d_offset)?);

        assert_eq!(7..11, resolved[3].actual_range);
        assert_eq!("202182159", resolved[3].to_string(d_offset)?);

        Ok(())
    }

    #[test]
    fn test_nested_struct() -> SimpleResult<()> {
        //           ----- hex ------  ----struct----
        //                            -A- -B- ---C---
        let data = b"\x00\x01\x02\x03\x41\x42\x43\x43\x01\x00\x00\x00".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        let t = H2Struct::new(vec![
            (
                "field_u32".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "struct".to_string(),
                H2Struct::new(vec![
                    ("A".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
                    ("B".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
                    ("C".to_string(), H2Number::new(SizedDefinition::U16(Endian::Big), SizedDisplay::Hex(Default::default())).into()),
                ])
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Decimal,
                )
            ),
        ]);

        assert_eq!(12, t.actual_size(s_offset)?);
        assert_eq!(12, t.actual_size(d_offset)?);

        let resolved = t.resolve_full(d_offset)?;
        assert_eq!(5, resolved.len());

        assert_eq!(0..4,         resolved[0].actual_range);
        assert_eq!("0x00010203", resolved[0].to_string(d_offset)?);
        //assert_eq!(vec!["field_u32".to_string()], resolved[0].field_name.unwrap());

        assert_eq!(4..5,     resolved[1].actual_range);
        assert_eq!("0x41",   resolved[1].to_string(d_offset)?);
        // assert_eq!(Some(vec!["struct".to_string(), "A".to_string()]), resolved[1].breadcrumbs);

        assert_eq!(5..6,     resolved[2].actual_range);
        assert_eq!("0x42",   resolved[2].to_string(d_offset)?);
        // assert_eq!(Some(vec!["struct".to_string(), "B".to_string()]), resolved[2].breadcrumbs);

        assert_eq!(6..8,     resolved[3].actual_range);
        assert_eq!("0x4343", resolved[3].to_string(d_offset)?);
        // assert_eq!(Some(vec!["struct".to_string(), "C".to_string()]), resolved[3].breadcrumbs);

        assert_eq!(8..12,    resolved[4].actual_range);
        assert_eq!("1",      resolved[4].to_string(d_offset)?);
        // assert_eq!(Some(vec!["field_u32_little".to_string()]), resolved[4].breadcrumbs);

        Ok(())
    }

    #[test]
    fn test_alignment() -> SimpleResult<()> {
        // P = padding / alignment bytes
        // Line 1: Starting at second byte, pad to 4 on both sides
        // Line 2: Pad end to 4 bytes
        // Line 3: The ZZZ are an array, then a 16-bit fully padded value: AA
        // Line 4: 4-byte big-endian value, padded to 8 bytes on the right
        // Line 4: 4-byte little-endian value, padded to 8 bytes on the right
        // Line 5: A 1-byte value to make sure we got to the right place
        let data = b"P\x01PP\
                     \x02PPP\
                     ZZZA\
                     APPP\
                     \x05\x06\x07\x08PPPP\
                     \x0c\x0b\x0a\x09PPPP\
                     \x0dPPP".to_vec();

        // Note: starting at offset 1 (so we can test the full alignment)
        let d_offset = ResolveOffset::Dynamic(Context::new_at(&data, 1));

        let t = H2Struct::new_aligned(Alignment::Loose(4), vec![
            (
                "field_u8_full".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(4),
                    SizedDefinition::U8,
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "field_u8_after".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(4),
                    SizedDefinition::U8,
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "padding_array".to_string(),
                H2Array::new(3, H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default()))),
            ),
            (
                "other_struct".to_string(),
                H2Struct::new(vec![
                    (
                        "nested_field_u16_full".to_string(),
                        H2Number::new_aligned(
                            Alignment::Loose(4),
                            SizedDefinition::U16(Endian::Big),
                            SizedDisplay::Hex(Default::default()),
                        )
                    ),
                ])
            ),
            (
                "u32_big".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(8),
                    SizedDefinition::U32(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "u32_little".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(8),
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "check".to_string(),
                H2Number::new_aligned(
                    Alignment::None,
                    SizedDefinition::U8,
                    SizedDisplay::Hex(Default::default()),
                )
            ),
        ]);

        // The sum of all the fields, with the fields' own alignment
        assert_eq!(32, t.actual_size(d_offset)?);
        assert_eq!(1..33, t.actual_range(d_offset)?);

        // We start offset by 1, which means to get to the next aligned size
        // (36), it would make this 35 long
        assert_eq!(35, t.aligned_size(d_offset)?);
        assert_eq!(1..36, t.aligned_range(d_offset)?);

        // 7 direct children - the fields
        assert_eq!(7, t.resolve_partial(d_offset)?.len());

        // 9 total children, due to the array
        assert_eq!(9, t.resolve_full(d_offset)?.len());

        let resolved = t.resolve_full(d_offset)?;

        assert_eq!("0x01", resolved[0].to_string(d_offset)?);
        assert_eq!(1..2,   resolved[0].actual_range);
        assert_eq!(1..4,   resolved[0].aligned_range);

        assert_eq!("0x02", resolved[1].to_string(d_offset)?);
        assert_eq!(4..5,   resolved[1].actual_range);
        assert_eq!(4..8,   resolved[1].aligned_range);

        assert_eq!("0x5a", resolved[2].to_string(d_offset)?);
        assert_eq!(8..9,   resolved[2].actual_range);
        assert_eq!(8..9,   resolved[2].aligned_range);

        assert_eq!("0x5a", resolved[3].to_string(d_offset)?);
        assert_eq!(9..10,  resolved[3].actual_range);
        assert_eq!(9..10,  resolved[3].aligned_range);

        assert_eq!("0x5a", resolved[4].to_string(d_offset)?);
        assert_eq!(10..11, resolved[4].actual_range);
        assert_eq!(10..11, resolved[4].aligned_range);

        assert_eq!("0x4141", resolved[5].to_string(d_offset)?);
        assert_eq!(11..13,   resolved[5].actual_range);
        assert_eq!(11..16,   resolved[5].aligned_range);

        assert_eq!("0x05060708", resolved[6].to_string(d_offset)?);
        assert_eq!(16..20,       resolved[6].actual_range);
        assert_eq!(16..24,       resolved[6].aligned_range);

        assert_eq!("0x090a0b0c", resolved[7].to_string(d_offset)?);
        assert_eq!(24..28,       resolved[7].actual_range);
        assert_eq!(24..32,       resolved[7].aligned_range);

        assert_eq!("0x0d", resolved[8].to_string(d_offset)?);
        assert_eq!(32..33, resolved[8].actual_range);
        assert_eq!(32..33, resolved[8].aligned_range);

        Ok(())
    }
}
