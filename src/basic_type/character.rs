use simple_error::{bail, SimpleResult};

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use sized_number::{Endian, Context};

use crate::{H2Type, H2Types, H2TypeTrait, Offset};
use crate::alignment::Alignment;

/// Configure whether invalid ASCII characters are an error or just replaced
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum StrictASCII {
    Strict,
    Permissive,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum CharacterType {
    ASCII(StrictASCII),
    UTF8,
    UTF16(Endian),
    UTF32(Endian),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Character {
    character_type: CharacterType,
}

impl Character {
    pub fn new_aligned(alignment: Alignment, character_type: CharacterType) -> H2Type {
        H2Type::new(alignment, H2Types::Character(Self {
            character_type: character_type
        }))
    }

    pub fn new(character_type: CharacterType) -> H2Type {
        Self::new_aligned(Alignment::None, character_type)
    }

    fn read_ascii_strict(context: Context) -> SimpleResult<char> {
        let number = context.read_u8()?;

        match number < 0x7F {
            true  => Ok(number as char),
            false => bail!("Invalid ASCII character: {:#x}", number),
        }
    }

    fn read_ascii_permissive(context: Context) -> SimpleResult<char> {
        let number = context.read_u8()?;

        match number < 0x7F {
            true  => Ok(number as char),
            false => Ok('�'),
        }
    }

    fn read_utf8(context: Context) -> SimpleResult<(u64, char)> {
        let (size, c) = context.read_utf8()?;

        Ok((size as u64, c))
    }

    fn read_utf16(context: Context, endian: Endian) -> SimpleResult<(u64, char)> {
        let (size, c) = context.read_utf16(endian)?;

        Ok((size as u64, c))
    }

    fn read_utf32(context: Context, endian: Endian) -> SimpleResult<char> {
        context.read_utf32(endian)
    }

    fn character(&self, context: Context) -> SimpleResult<char> {
        match self.character_type {
            CharacterType::ASCII(strict)   => {
                match strict {
                    StrictASCII::Strict     => Ok(Self::read_ascii_strict(context)?),
                    StrictASCII::Permissive => Ok(Self::read_ascii_permissive(context)?),
                }
            },
            CharacterType::UTF8          => {
                Ok(Self::read_utf8(context)?.1)
            },
            CharacterType::UTF16(endian) => {
                Ok(Self::read_utf16(context, endian)?.1)
            },
            CharacterType::UTF32(endian) => {
                Ok(Self::read_utf32(context, endian)?)
            }
        }
    }
}

impl H2TypeTrait for Character {
    fn is_static(&self) -> bool {
        match self.character_type {
            CharacterType::ASCII(_)    => true,
            CharacterType::UTF8        => false,
            CharacterType::UTF16(_)    => false,
            CharacterType::UTF32(_)    => true
        }
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        match self.character_type {
            CharacterType::ASCII(_)      => Ok(1),
            CharacterType::UTF8          => Ok(Self::read_utf8(offset.get_dynamic()?)?.0),
            CharacterType::UTF16(endian) => Ok(Self::read_utf16(offset.get_dynamic()?, endian)?.0),
            CharacterType::UTF32(_)      => Ok(4)
        }
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => {
                match self.character_type {
                    CharacterType::ASCII(_)    => Ok("ASCII Character".to_string()),
                    CharacterType::UTF8        => Ok("UTF-8 Character".to_string()),
                    CharacterType::UTF16(_)    => Ok("UTF-16 Character".to_string()),
                    CharacterType::UTF32(_)    => Ok("UTF-32 Character".to_string()),
                }
            },
            Offset::Dynamic(context) => {
                let c = self.character(context)?;

                match c as u32 {
                    0x00        => Ok("'\\0'".to_string()),
                    0x01..=0x06 => Ok(format!("'\\x{:02x}'", c as u32)),
                    0x07        => Ok("'\\a'".to_string()),
                    0x08        => Ok("'\\b'".to_string()),
                    0x09        => Ok("'\\t'".to_string()),
                    0x0a        => Ok("'\\n'".to_string()),
                    0x0b        => Ok("'\\v'".to_string()),
                    0x0c        => Ok("'\\f'".to_string()),
                    0x0d        => Ok("'\\r'".to_string()),
                    0x0e..=0x1f => Ok(format!("'\\x{:02x}'", c as u32)),
                    _ => Ok(format!("'{}'", self.character(context)?))
                }
            }
        }
    }

    fn to_char(&self, offset: Offset) -> SimpleResult<char> {
        self.character(offset.get_dynamic()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    #[test]
    fn test_ascii_type_unaligned() -> SimpleResult<()> {
        let c = Character::new(CharacterType::ASCII(StrictASCII::Permissive));

        assert_eq!(true, c.is_static());

        assert_eq!(1, c.actual_size(Offset::Static(0))?);
        assert_eq!(0..1, c.actual_range(Offset::Static(0))?);

        assert_eq!(1, c.aligned_size(Offset::Static(0))?);
        assert_eq!(0..1, c.aligned_range(Offset::Static(0))?);

        assert_eq!(0, c.children(Offset::Static(0))?.len());
        assert_eq!(0, c.related(Offset::Static(0))?.len());

        Ok(())
    }

    #[test]
    fn test_ascii_resolve() -> SimpleResult<()> {
        let data = b"\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let r = Character::new(CharacterType::ASCII(StrictASCII::Permissive)).resolve(offset, None)?;
        assert_eq!(1, r.actual_size());
        assert_eq!(0..1, r.actual_range);

        assert_eq!(1, r.aligned_size());
        assert_eq!(0..1, r.aligned_range);

        assert_eq!(0, r.children.len());
        assert_eq!(0, r.related.len());
        assert_eq!("'A'", r.value);

        Ok(())
    }

    #[test]
    fn test_ascii_type_aligned() -> SimpleResult<()> {
        let c = Character::new_aligned(Alignment::Loose(4), CharacterType::ASCII(StrictASCII::Permissive));

        assert_eq!(true, c.is_static());

        assert_eq!(1, c.actual_size(Offset::Static(0))?);
        assert_eq!(0..1, c.actual_range(Offset::Static(0))?);

        assert_eq!(4, c.aligned_size(Offset::Static(0))?);
        assert_eq!(0..4, c.aligned_range(Offset::Static(0))?);

        assert_eq!(0, c.children(Offset::Static(0))?.len());
        assert_eq!(0, c.related(Offset::Static(0))?.len());

        Ok(())
    }

    #[test]
    fn test_ascii_resolve_aligned() -> SimpleResult<()> {
        let data = b"\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let r = Character::new_aligned(Alignment::Loose(4), CharacterType::ASCII(StrictASCII::Permissive)).resolve(offset, None)?;
        assert_eq!(1, r.actual_size());
        assert_eq!(0..1, r.actual_range);

        assert_eq!(4, r.aligned_size());
        assert_eq!(0..4, r.aligned_range);

        assert_eq!(0, r.children.len());
        assert_eq!(0, r.related.len());
        assert_eq!("'A'", r.value);

        Ok(())
    }

    #[test]
    fn test_ascii_to_string_permissive() -> SimpleResult<()> {
        let data = b"\x00\x06\x20\x41\x42\x7e\x7f\x80\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));
        let t = Character::new(CharacterType::ASCII(StrictASCII::Permissive));

        assert_eq!("'\\0'",   t.to_string(offset.at(0))?);
        assert_eq!("'\\x06'", t.to_string(offset.at(1))?);
        assert_eq!("' '", t.to_string(offset.at(2))?);
        assert_eq!("'A'", t.to_string(offset.at(3))?);
        assert_eq!("'B'", t.to_string(offset.at(4))?);
        assert_eq!("'~'", t.to_string(offset.at(5))?);
        assert_eq!("'�'", t.to_string(offset.at(6))?);
        assert_eq!("'�'", t.to_string(offset.at(7))?);
        assert_eq!("'�'", t.to_string(offset.at(8))?);

        Ok(())
    }

    #[test]
    fn test_ascii_to_string_strict() -> SimpleResult<()> {
        let data = b"\x00\x06\x20\x41\x42\x7e\x7f\x80\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));
        let t = Character::new(CharacterType::ASCII(StrictASCII::Strict));

        assert!(t.to_string(offset.at(6)).is_err());
        assert!(t.to_string(offset.at(7)).is_err());
        assert!(t.to_string(offset.at(8)).is_err());

        assert_eq!("'\\0'",   t.to_string(offset.at(0))?);
        assert_eq!("'\\x06'", t.to_string(offset.at(1))?);
        assert_eq!("' '",     t.to_string(offset.at(2))?);
        assert_eq!("'A'",     t.to_string(offset.at(3))?);
        assert_eq!("'B'",     t.to_string(offset.at(4))?);
        assert_eq!("'~'",     t.to_string(offset.at(5))?);

        Ok(())
    }

    #[test]
    fn test_utf8_size() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!(1, Character::new(CharacterType::UTF8).actual_size(offset.at(0))?);
        assert_eq!(1, Character::new(CharacterType::UTF8).actual_size(offset.at(1))?);
        assert_eq!(3, Character::new(CharacterType::UTF8).actual_size(offset.at(2))?);
        assert_eq!(3, Character::new(CharacterType::UTF8).actual_size(offset.at(5))?);
        assert_eq!(4, Character::new(CharacterType::UTF8).actual_size(offset.at(8))?);
        assert_eq!(4, Character::new(CharacterType::UTF8).actual_size(offset.at(12))?);
        assert_eq!(2, Character::new(CharacterType::UTF8).actual_size(offset.at(16))?);

        Ok(())
    }

    #[test]
    fn test_utf8_to_string() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("'A'", Character::new(CharacterType::UTF8).to_string(offset.at(0))?);
        assert_eq!("'B'", Character::new(CharacterType::UTF8).to_string(offset.at(1))?);
        assert_eq!("'❄'", Character::new(CharacterType::UTF8).to_string(offset.at(2))?);
        assert_eq!("'☢'", Character::new(CharacterType::UTF8).to_string(offset.at(5))?);
        assert_eq!("'𝄞'", Character::new(CharacterType::UTF8).to_string(offset.at(8))?);
        assert_eq!("'😈'", Character::new(CharacterType::UTF8).to_string(offset.at(12))?);
        assert_eq!("'÷'", Character::new(CharacterType::UTF8).to_string(offset.at(16))?);

        Ok(())
    }

    #[test]
    fn test_utf8_too_short() -> SimpleResult<()> {
        let data = b"\xE2".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert!(Character::new(CharacterType::UTF8).to_string(offset.at(0)).is_err());
        assert!(Character::new(CharacterType::UTF8).to_string(offset.at(1)).is_err());

        Ok(())
    }

    #[test]
    fn test_utf16_size_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!(2, Character::new(CharacterType::UTF16(Endian::Big)).actual_size(offset.at(0))?);
        assert_eq!(2, Character::new(CharacterType::UTF16(Endian::Big)).actual_size(offset.at(2))?);
        assert_eq!(2, Character::new(CharacterType::UTF16(Endian::Big)).actual_size(offset.at(4))?);
        assert_eq!(2, Character::new(CharacterType::UTF16(Endian::Big)).actual_size(offset.at(6))?);

        // Double
        assert_eq!(4, Character::new(CharacterType::UTF16(Endian::Big)).actual_size(offset.at(8))?);
        assert_eq!(4, Character::new(CharacterType::UTF16(Endian::Big)).actual_size(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_utf16_to_string_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!("'A'", Character::new(CharacterType::UTF16(Endian::Big)).to_string(offset.at(0))?);
        assert_eq!("'B'", Character::new(CharacterType::UTF16(Endian::Big)).to_string(offset.at(2))?);
        assert_eq!("'❄'", Character::new(CharacterType::UTF16(Endian::Big)).to_string(offset.at(4))?);
        assert_eq!("'☢'", Character::new(CharacterType::UTF16(Endian::Big)).to_string(offset.at(6))?);

        // Double
        assert_eq!("'𝄞'", Character::new(CharacterType::UTF16(Endian::Big)).to_string(offset.at(8))?);
        assert_eq!("'😈'", Character::new(CharacterType::UTF16(Endian::Big)).to_string(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_utf16_to_string_little_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x41\x00\x42\x00\x44\x27\x22\x26\x34\xd8\x1e\xdd\x3d\xd8\x08\xde".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!("'A'", Character::new(CharacterType::UTF16(Endian::Little)).to_string(offset.at(0))?);
        assert_eq!("'B'", Character::new(CharacterType::UTF16(Endian::Little)).to_string(offset.at(2))?);
        assert_eq!("'❄'", Character::new(CharacterType::UTF16(Endian::Little)).to_string(offset.at(4))?);
        assert_eq!("'☢'", Character::new(CharacterType::UTF16(Endian::Little)).to_string(offset.at(6))?);

        // Double
        assert_eq!("'𝄞'", Character::new(CharacterType::UTF16(Endian::Little)).to_string(offset.at(8))?);
        assert_eq!("'😈'", Character::new(CharacterType::UTF16(Endian::Little)).to_string(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_utf32_to_string_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x41\x00\x00\x00\x42\x00\x00\x27\x44\x00\x00\x26\x22\x00\x01\xD1\x1E\x00\x01\xF6\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("'A'", Character::new(CharacterType::UTF32(Endian::Big)).to_string(offset.at(0))?);
        assert_eq!("'B'", Character::new(CharacterType::UTF32(Endian::Big)).to_string(offset.at(4))?);
        assert_eq!("'❄'", Character::new(CharacterType::UTF32(Endian::Big)).to_string(offset.at(8))?);
        assert_eq!("'☢'", Character::new(CharacterType::UTF32(Endian::Big)).to_string(offset.at(12))?);
        assert_eq!("'𝄞'", Character::new(CharacterType::UTF32(Endian::Big)).to_string(offset.at(16))?);
        assert_eq!("'😈'", Character::new(CharacterType::UTF32(Endian::Big)).to_string(offset.at(20))?);

        Ok(())
    }

    #[test]
    fn test_utf32_to_string_little_endian() -> SimpleResult<()> {
        let data = b"\x41\x00\x00\x00\x42\x00\x00\x00\x44\x27\x00\x00\x22\x26\x00\x00\x1E\xd1\x01\x00\x08\xf6\x01\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("'A'", Character::new(CharacterType::UTF32(Endian::Little)).to_string(offset.at(0))?);
        assert_eq!("'B'", Character::new(CharacterType::UTF32(Endian::Little)).to_string(offset.at(4))?);
        assert_eq!("'❄'", Character::new(CharacterType::UTF32(Endian::Little)).to_string(offset.at(8))?);
        assert_eq!("'☢'", Character::new(CharacterType::UTF32(Endian::Little)).to_string(offset.at(12))?);
        assert_eq!("'𝄞'", Character::new(CharacterType::UTF32(Endian::Little)).to_string(offset.at(16))?);
        assert_eq!("'😈'", Character::new(CharacterType::UTF32(Endian::Little)).to_string(offset.at(20))?);

        Ok(())
    }

    #[test]
    fn test_null() -> SimpleResult<()> {
        let data = b"\x41\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!('A',  Character::new(CharacterType::ASCII(StrictASCII::Permissive)).to_char(offset.at(0))?);
        assert_eq!('\0', Character::new(CharacterType::ASCII(StrictASCII::Permissive)).to_char(offset.at(1))?);
        assert_eq!('A',  Character::new(CharacterType::ASCII(StrictASCII::Permissive)).to_char(offset.at(2))?);

        assert_eq!('A',  Character::new(CharacterType::ASCII(StrictASCII::Strict)).to_char(offset.at(0))?);
        assert_eq!('\0', Character::new(CharacterType::ASCII(StrictASCII::Strict)).to_char(offset.at(1))?);
        assert_eq!('A',  Character::new(CharacterType::ASCII(StrictASCII::Strict)).to_char(offset.at(2))?);

        let data = b"\x41\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!('A',  Character::new(CharacterType::UTF8).to_char(offset.at(0))?);
        assert_eq!('\0', Character::new(CharacterType::UTF8).to_char(offset.at(1))?);
        assert_eq!('A',  Character::new(CharacterType::UTF8).to_char(offset.at(2))?);

        let data = b"\x00\x41\x00\x00\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!('A',  Character::new(CharacterType::UTF16(Endian::Big)).to_char(offset.at(0))?);
        assert_eq!('\0', Character::new(CharacterType::UTF16(Endian::Big)).to_char(offset.at(2))?);
        assert_eq!('A',  Character::new(CharacterType::UTF16(Endian::Big)).to_char(offset.at(4))?);

        let data = b"\x00\x00\x00\x41\x00\x00\x00\x00\x00\x00\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!('A',  Character::new(CharacterType::UTF32(Endian::Big)).to_char(offset.at(0))?);
        assert_eq!('\0', Character::new(CharacterType::UTF32(Endian::Big)).to_char(offset.at(4))?);
        assert_eq!('A',  Character::new(CharacterType::UTF32(Endian::Big)).to_char(offset.at(8))?);

        Ok(())
    }
}
