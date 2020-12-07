use simple_error::SimpleResult;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use sized_number::Endian;

use crate::{H2Type, H2Types, H2TypeTrait, Offset};
use crate::alignment::Alignment;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct UTF16 {
    endian: Endian,
}

impl UTF16 {
    pub fn new_aligned(alignment: Alignment, endian: Endian) -> H2Type {
        H2Type::new(alignment, H2Types::UTF16(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(Alignment::None, endian)
    }
}

impl H2TypeTrait for UTF16 {
    fn is_static(&self) -> bool {
        false
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        let context = offset.get_dynamic()?;

        Ok(context.read_utf16(self.endian)?.0 as u64)
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        let context = offset.get_dynamic()?;

        Ok(context.read_utf16(self.endian)?.1.to_string())
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
        assert_eq!(2, UTF16::new(Endian::Big).actual_size(offset.at(0))?);
        assert_eq!(2, UTF16::new(Endian::Big).actual_size(offset.at(2))?);
        assert_eq!(2, UTF16::new(Endian::Big).actual_size(offset.at(4))?);
        assert_eq!(2, UTF16::new(Endian::Big).actual_size(offset.at(6))?);

        // Double
        assert_eq!(4, UTF16::new(Endian::Big).actual_size(offset.at(8))?);
        assert_eq!(4, UTF16::new(Endian::Big).actual_size(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_to_string_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!("A", UTF16::new(Endian::Big).to_string(offset.at(0))?);
        assert_eq!("B", UTF16::new(Endian::Big).to_string(offset.at(2))?);
        assert_eq!("❄", UTF16::new(Endian::Big).to_string(offset.at(4))?);
        assert_eq!("☢", UTF16::new(Endian::Big).to_string(offset.at(6))?);

        // Double
        assert_eq!("𝄞", UTF16::new(Endian::Big).to_string(offset.at(8))?);
        assert_eq!("😈", UTF16::new(Endian::Big).to_string(offset.at(12))?);

        Ok(())
    }

    #[test]
    fn test_to_string_little_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x41\x00\x42\x00\x44\x27\x22\x26\x34\xd8\x1e\xdd\x3d\xd8\x08\xde".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        // Single
        assert_eq!("A", UTF16::new(Endian::Little).to_string(offset.at(0))?);
        assert_eq!("B", UTF16::new(Endian::Little).to_string(offset.at(2))?);
        assert_eq!("❄", UTF16::new(Endian::Little).to_string(offset.at(4))?);
        assert_eq!("☢", UTF16::new(Endian::Little).to_string(offset.at(6))?);

        // Double
        assert_eq!("𝄞", UTF16::new(Endian::Little).to_string(offset.at(8))?);
        assert_eq!("😈", UTF16::new(Endian::Little).to_string(offset.at(12))?);

        Ok(())
    }
}
