use simple_error::{bail, SimpleResult};

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2Types, ResolvedType, H2TypeTrait, Offset};
use crate::alignment::Alignment;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct H2Struct {
    fields: Vec<(String, H2Type)>,
}

impl H2Struct {
    // TODO: We need to prevent zero-length arrays
    pub fn new_aligned(alignment: Alignment, fields: Vec<(String, H2Type)>) -> SimpleResult<H2Type> {
        if fields.len() == 0 {
            bail!("Structs must contain at least one field");
        }

        Ok(H2Type::new(alignment, H2Types::H2Struct(Self {
            fields: fields
        })))
    }

    pub fn new(fields: Vec<(String, H2Type)>) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, fields)
    }
}

impl H2TypeTrait for H2Struct {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool {
        // Loop over each field - return an object as soon as is_static() is
        // false
        self.fields.iter().find(|(_, t)| {
            t.is_static() == false
        }).is_none()
    }

    // I think the default implementation will work fine
    // fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
    //     let resolved = self.resolve_partial(offset)?;

    //     if let Some(first) = resolved.first() {
    //         if let Some(last) = resolved.last() {
    //             return Ok(last.aligned_range.end - first.aligned_range.start);
    //         } else {
    //             bail!("No elements");
    //         }
    //     } else {
    //         bail!("No elements");
    //     }
    // }

    fn children(&self, _offset: Offset) -> SimpleResult<Vec<H2Type>> {
        Ok(self.fields.iter().map(|(_name, field_type)| {
            field_type.clone()
        }).collect())
    }

    // Get the user-facing name of the type
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
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};

    use crate::basic_type::{H2Number, ASCII, StrictASCII, IPv4};
    use crate::complex_type::H2Array;

    #[test]
    fn test_struct() -> SimpleResult<()> {
        //           ----- hex ------ --hex-- -o-    ----decimal----
        let data = b"\x00\x01\x02\x03\x00\x01p\x0fppp\x0f\x0e\x0d\x0c".to_vec();

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
                H2Number::new_aligned(
                    Alignment::Loose(3),
                    SizedDefinition::U16(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "field_u8".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(4),
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
        ])?;

        // Use real data
        let offset = Offset::Dynamic(Context::new(&data));
        assert_eq!(true, t.is_static());
        assert_eq!(15, t.actual_size(offset)?);
        assert_eq!(15, t.aligned_size(offset)?);
        assert_eq!(0..15, t.actual_range(offset)?);
        assert_eq!(0..15, t.aligned_range(offset)?);
        // TODO: This needs field names
        assert_eq!("[0x00010203, 0x0001, 0o17, 202182159]", t.to_string(offset)?);
        assert_eq!(0, t.related(offset)?.len());
        assert_eq!(4, t.children(offset)?.len());

        // Resolve and validate the resolved version
        let r = t.resolve(offset)?;
        assert_eq!(15, r.actual_size());
        assert_eq!(15, r.aligned_size());
        assert_eq!(0..15, r.actual_range);
        assert_eq!(0..15, r.aligned_range);
        // TODO: This needs field names
        assert_eq!("[0x00010203, 0x0001, 0o17, 202182159]", r.value);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Use abstract data
        let offset = Offset::Static(0);
        assert_eq!(true, t.is_static());
        assert_eq!(15, t.actual_size(offset)?);
        assert_eq!(15, t.aligned_size(offset)?);
        assert_eq!(0..15, t.actual_range(offset)?);
        assert_eq!(0..15, t.aligned_range(offset)?);
        // TODO: This needs field names
        assert_eq!("[Number, Number, Number, Number]", t.to_string(offset)?);
        assert_eq!(0, t.related(offset)?.len());
        assert_eq!(4, t.children(offset)?.len());

        // Resolve and validate the resolved version
        let r = t.resolve(offset)?;
        assert_eq!(15, r.actual_size());
        assert_eq!(15, r.aligned_size());
        assert_eq!(0..15, r.actual_range);
        assert_eq!(0..15, r.aligned_range);
        // TODO: This needs field names
        assert_eq!("[Number, Number, Number, Number]", r.value);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        Ok(())
    }

    #[test]
    fn test_nested_struct() -> SimpleResult<()> {
        //              -- hex --  ----------------struct----------------  ----- ipv4 ----
        //                         -A- -B- ---C--- ----- char_array -----
        let data = b"XXX\x00\x01pp\x41\x42\x43\x43\x61\x62\x63\x64\x65ppp\x7f\x00\x00\x01".to_vec();

        let t = H2Struct::new(vec![
            (
                "hex".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(4),
                    SizedDefinition::U16(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "struct".to_string(),
                H2Struct::new(vec![
                    (
                        "A".to_string(),
                        H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()
                    ),
                    (
                        "B".to_string(),
                        H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()
                    ),
                    (
                        "C".to_string(),
                        H2Number::new(SizedDefinition::U16(Endian::Big), SizedDisplay::Hex(Default::default())).into()
                    ),
                    (
                        "char_array".to_string(),
                        H2Array::new_aligned(
                            Alignment::Loose(8),
                            5,
                            ASCII::new(StrictASCII::Permissive),
                        )?,
                    )
                ])?,
            ),
            (
                "ipv4".to_string(),
                IPv4::new(Endian::Big)
            ),
        ])?;

        // Start at 3 to test offsets and alignment
        let offset = Offset::Dynamic(Context::new_at(&data, 3));
        assert_eq!(true, t.is_static());
        assert_eq!(20, t.actual_size(offset)?);
        assert_eq!(20, t.aligned_size(offset)?);
        assert_eq!(3..23, t.actual_range(offset)?);
        assert_eq!(3..23, t.aligned_range(offset)?);
        // TODO: This needs field names
        assert_eq!("[0x0001, [0x41, 0x42, 0x4343, [a, b, c, d, e]], 127.0.0.1]", t.to_string(offset)?);
        assert_eq!(0, t.related(offset)?.len());
        assert_eq!(3, t.children(offset)?.len());

        // Make sure it resolves sanely
        let r = t.resolve(offset)?;
        assert_eq!(20, r.actual_size());
        assert_eq!(20, r.aligned_size());
        assert_eq!(3..23, r.actual_range);
        assert_eq!(3..23, r.aligned_range);
        // TODO: This needs field names
        assert_eq!("[0x0001, [0x41, 0x42, 0x4343, [a, b, c, d, e]], 127.0.0.1]", r.value);
        assert_eq!(0, r.related.len());
        assert_eq!(3, r.children.len());

        // Check the first child
        assert_eq!(2, r.children[0].actual_size());
        assert_eq!(4, r.children[0].aligned_size());
        assert_eq!("0x0001", r.children[0].value);
        assert_eq!(0, r.children[0].children.len());

        // Check the second child
        assert_eq!(12, r.children[1].actual_size());
        assert_eq!(12, r.children[1].aligned_size());
        assert_eq!("[0x41, 0x42, 0x4343, [a, b, c, d, e]]", r.children[1].value);
        assert_eq!(4, r.children[1].children.len());

        // Check the character array
        assert_eq!(5, r.children[1].children[3].actual_size());
        assert_eq!(8, r.children[1].children[3].aligned_size());
        assert_eq!(5, r.children[1].children[3].children.len());

        Ok(())
    }

    #[test]
    fn test_dynamically_sized_struct() -> SimpleResult<()> {
        // TODO
        Ok(())
    }
}
