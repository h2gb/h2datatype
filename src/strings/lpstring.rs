use simple_error::SimpleResult;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2Types, H2TypeTrait, Offset, Alignment};
use crate::complex_type::H2Array;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct LPString {
    length: Box<H2Type>,
    character: Box<H2Type>,
}

impl LPString {
    pub fn new_aligned(alignment: Alignment, length: H2Type, character: H2Type) -> H2Type {
        H2Type::new(alignment, H2Types::LPString(Self {
            length: Box::new(length),
            character: Box::new(character),
        }))
    }

    pub fn new(length: H2Type, character: H2Type) -> H2Type {
        Self::new_aligned(Alignment::None, length, character)
    }

    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<char>)> {
        let length = self.length.to_u64(offset)?;

        let mut position = offset.position() + self.length.aligned_size(offset)?;

        let mut result = Vec::new();
        for _ in 0..length {
            let this_offset = offset.at(position);
            let this_size = self.character.actual_size(this_offset)?;
            let this_character = self.character.to_char(this_offset)?;

            result.push(this_character);
            position = position + this_size;
        }

        Ok((position, result))
    }
}

impl H2TypeTrait for LPString {
    fn is_static(&self) -> bool {
        self.character.is_static()
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        Ok(self.analyze(offset)?.0)
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        // Get the length so we can truncate
        let (_, chars) = self.analyze(offset)?;

        // Strip the last character (which is the null byte)
        let s: String = chars.into_iter().collect();

        Ok(s)
    }

    fn children(&self, offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        let length = self.length.to_u64(offset)?;

        Ok(vec![
            ( Some("size".to_string()), self.length.as_ref().clone() ),
            ( None,                     H2Array::new(length, self.character.as_ref().clone())? ),
        ])

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};
    use crate::basic_type::{Character, CharacterType, H2Number};
    use crate::Alignment;

    #[test]
    fn test_utf8_lpstring() -> SimpleResult<()> {
        //                     --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x00\x07\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let size_type = H2Number::new(SizedDefinition::U16(Endian::Big), SizedDisplay::Decimal);

        let a = LPString::new(size_type, Character::new(CharacterType::UTF8));
        assert_eq!("AB❄☢𝄞😈÷", a.to_string(offset)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_lpstring() -> SimpleResult<()> {
        let data = b"\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let size_type = H2Number::new(SizedDefinition::U8, SizedDisplay::Decimal);
        let a = LPString::new(size_type, Character::new(CharacterType::UTF8));
        assert_eq!("", a.to_string(offset)?);

        Ok(())
    }

    #[test]
    fn test_blank_lpstring() -> SimpleResult<()> {
        let data = b"".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let size_type = H2Number::new(SizedDefinition::U8, SizedDisplay::Decimal);
        let a = LPString::new(size_type, Character::new(CharacterType::UTF8));
        assert!(a.to_string(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_aligned_length_lpstring() -> SimpleResult<()> {
        let data = b"\x00\x07PPPPPP\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let size_type = H2Number::new_aligned(Alignment::Loose(8), SizedDefinition::U16(Endian::Big), SizedDisplay::Decimal);

        let a = LPString::new(size_type, Character::new(CharacterType::UTF8));
        assert_eq!("AB❄☢𝄞😈÷", a.to_string(offset)?);

        Ok(())
    }

    #[test]
    fn test_utf8_to_array() -> SimpleResult<()> {
        //                 --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x07\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let size_type = H2Number::new(SizedDefinition::U8, SizedDisplay::Decimal);
        let a: H2Type = LPString::new(size_type, Character::new(CharacterType::UTF8));
        let array = a.resolve(offset, None)?;

        // Should just have two children - the length and the array
        assert_eq!(2, array.children.len());

        // The first child should just be the length
        assert_eq!("7", array.children[0].value);

        // The second child should be an array of the characters
        assert_eq!("[ 'A', 'B', '❄', '☢', '𝄞', '😈', '÷' ]", array.children[1].value);
        assert_eq!(7, array.children[1].children.len());

        Ok(())
    }
}
