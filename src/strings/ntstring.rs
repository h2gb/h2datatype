use simple_error::{bail, SimpleResult};

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2Types, H2TypeTrait, Offset, Alignment};
use crate::complex_type::H2Array;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct NTString {
    character: Box<H2Type>,
}

impl NTString {
    pub fn new_aligned(alignment: Alignment, character: H2Type) -> H2Type {
        H2Type::new(alignment, H2Types::NTString(Self {
            character: Box::new(character),
        }))
    }

    pub fn new(character: H2Type) -> H2Type {
        Self::new_aligned(Alignment::None, character)
    }

    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<char>)> {
        let mut position = offset.position();
        let mut result = Vec::new();

        loop {
            let this_offset = offset.at(position);
            let this_size = self.character.aligned_size(this_offset)?;
            let this_character = self.character.to_char(this_offset)?;

            result.push(this_character);
            position = position + this_size;

            if this_character == '\0' {
                break;
            }
        }

        Ok((position, result))
    }
}

impl H2TypeTrait for NTString {
    fn is_static(&self) -> bool {
        self.character.is_static()
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        Ok(self.analyze(offset)?.0)
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        // Get the length so we can truncate
        let (_, chars) = self.analyze(offset)?;

        if chars.len() == 0 {
            return Ok("".to_string());
        }

        // Strip the last character (which is the null byte)
        let s: String = chars[0..(chars.len() - 1)].into_iter().collect();

        Ok(s)
    }

    fn children(&self, offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        bail!("TODO");
        // let (size, _) = self.analyze(offset)?;

        // let array = H2Array::new(size, self.character.clone())?;

        // Ok(vec![(None, array)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;
    use crate::basic_type::{Character, CharacterType};
    use crate::Alignment;

    #[test]
    fn test_utf8_string() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(Character::new(CharacterType::UTF8));
        assert_eq!("AB❄☢𝄞😈÷", a.to_string(offset)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_utf8_string() -> SimpleResult<()> {
        let data = b"\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(Character::new(CharacterType::UTF8));
        assert_eq!("", a.to_string(offset)?);

        Ok(())
    }

    #[test]
    fn test_blank_string() -> SimpleResult<()> {
        let data = b"".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(Character::new(CharacterType::UTF8));
        assert!(a.to_string(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_missing_terminator() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(Character::new(CharacterType::UTF8));
        assert!(a.to_string(offset).is_err());

        Ok(())
    }

    #[test]
    fn test_utf8_aligned_characters_string() -> SimpleResult<()> {
        // We're aligning to 3-byte characters, so 1, 2, and 4 byte characters
        // get padded
        //             --    --    ----------  ----------  --------------    --------------    ------
        let data = b"\x41PP\x42PP\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9EPP\xF0\x9F\x98\x88PP\xc3\xb7P\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(Character::new_aligned(Alignment::Loose(3), CharacterType::UTF8));
        assert_eq!("AB❄☢𝄞😈÷", a.to_string(offset)?);

        Ok(())
    }

    // #[test]
    // fn test_utf8_to_array() -> SimpleResult<()> {
    //     //             --  --  ----------  ----------  --------------  --------------  ------
    //     let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
    //     let offset = Offset::Dynamic(Context::new(&data));

    //     let a: H2Type = NTString::new(Character::new(CharacterType::UTF8));
    //     let array = a.resolve(offset, None)?;

    //     println!("{:?}", array);

    //     Ok(())
    // }
}