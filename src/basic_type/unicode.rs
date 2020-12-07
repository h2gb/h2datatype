use simple_error::{SimpleResult, bail};
use std::char;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use sized_number::{Endian, Context};

use crate::{H2Type, H2Types, H2TypeTrait, Offset};
use crate::alignment::Alignment;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Unicode {
    endian: Endian,
}

impl Unicode {
    pub fn new_aligned(alignment: Alignment, endian: Endian) -> H2Type {
        H2Type::new(alignment, H2Types::Unicode(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(Alignment::None, endian)
    }

    fn read_unicode16(&self, context: Context) -> SimpleResult<char> {
        // Just read one number
        let numbers = vec![
            context.read_u16(self.endian)?,
        ];

        if let Ok(s) = String::from_utf16(&numbers) {
            if let Some(c) = s.chars().next() {
                return Ok(c);
            }
        }

        bail!("Failed to parse unicode character");
    }

    fn read_unicode32(&self, context: Context) -> SimpleResult<char> {
        let numbers = vec![
            context.read_u16(self.endian)?,
            context.at(context.position() + 2).read_u16(self.endian)?,
        ];

        if let Ok(s) = String::from_utf16(&numbers) {
            if let Some(c) = s.chars().next() {
                return Ok(c);
            }
        }

        bail!("Failed to parse unicode character");
    }

    fn read_unicode(&self, offset: Offset) -> SimpleResult<(u64, char)> {
        let context = offset.get_dynamic()?;

        // Try 16 bits first
        if let Ok(c) = self.read_unicode16(context) {
            return Ok((2, c));
        }

        // If that fails, commit to 32 bits
        Ok((4, self.read_unicode32(context)?))
    }
}

impl H2TypeTrait for Unicode {
    fn is_static(&self) -> bool {
        //f0 9d 84 9e Unicode requires context
        false
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        let (size, _) = self.read_unicode(offset)?;

        Ok(size)
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        let (_, c) = self.read_unicode(offset)?;

        Ok(format!("{}", c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    #[test]
    fn test_size_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!(2, Unicode::new(Endian::Big).actual_size(offset.at(0))?);
        assert_eq!(2, Unicode::new(Endian::Big).actual_size(offset.at(2))?);
        assert_eq!(2, Unicode::new(Endian::Big).actual_size(offset.at(4))?);
        assert_eq!(2, Unicode::new(Endian::Big).actual_size(offset.at(6))?);

        // Double
        assert_eq!(4, Unicode::new(Endian::Big).actual_size(offset.at(8))?);
        assert_eq!(4, Unicode::new(Endian::Big).actual_size(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_to_string_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!("A", Unicode::new(Endian::Big).to_string(offset.at(0))?);
        assert_eq!("B", Unicode::new(Endian::Big).to_string(offset.at(2))?);
        assert_eq!("❄", Unicode::new(Endian::Big).to_string(offset.at(4))?);
        assert_eq!("☢", Unicode::new(Endian::Big).to_string(offset.at(6))?);

        // Double
        assert_eq!("𝄞", Unicode::new(Endian::Big).to_string(offset.at(8))?);
        assert_eq!("😈", Unicode::new(Endian::Big).to_string(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_to_string_little_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x41\x00\x42\x00\x44\x27\x22\x26\x34\xd8\x1e\xdd\x3d\xd8\x08\xde".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!("A", Unicode::new(Endian::Little).to_string(offset.at(0))?);
        assert_eq!("B", Unicode::new(Endian::Little).to_string(offset.at(2))?);
        assert_eq!("❄", Unicode::new(Endian::Little).to_string(offset.at(4))?);
        assert_eq!("☢", Unicode::new(Endian::Little).to_string(offset.at(6))?);

        // Double
        assert_eq!("𝄞", Unicode::new(Endian::Little).to_string(offset.at(8))?);
        assert_eq!("😈", Unicode::new(Endian::Little).to_string(offset.at(12))?);

        Ok(())
    }
}
