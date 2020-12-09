use simple_error::SimpleResult;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2TypeTrait, Offset};
use crate::complex_type::H2Array;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct LPString {
    length: H2Type,
    character: H2Type,
}

impl LPString {
    // TODO: Handle 0-length
    pub fn new(length: H2Type, character: H2Type) -> Self {
        Self {
            length: length,
            character: character,
        }
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
        let (size, _) = self.analyze(offset)?;

        let array = H2Array::new(size, self.character.clone())?;

        Ok(vec![(None, array)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};
    use crate::basic_type::{Character, CharacterType, H2Number};

    #[test]
    fn test_utf8_lpstring() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
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

    // TODO: test an aligned size
}
