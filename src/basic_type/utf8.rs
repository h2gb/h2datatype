use simple_error::SimpleResult;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2Types, H2TypeTrait, Offset};
use crate::alignment::Alignment;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct UTF8 {
}

impl UTF8 {
    pub fn new_aligned(alignment: Alignment) -> H2Type {
        H2Type::new(alignment, H2Types::UTF8(Self {
        }))
    }

    pub fn new() -> H2Type {
        Self::new_aligned(Alignment::None)
    }
}

impl H2TypeTrait for UTF8 {
    fn is_static(&self) -> bool {
        false
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        let context = offset.get_dynamic()?;

        Ok(context.read_utf8()?.0 as u64)
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        let context = offset.get_dynamic()?;

        Ok(context.read_utf8()?.1.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    #[test]
    fn test_size() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!(1, UTF8::new().actual_size(offset.at(0))?);
        assert_eq!(1, UTF8::new().actual_size(offset.at(1))?);
        assert_eq!(3, UTF8::new().actual_size(offset.at(2))?);
        assert_eq!(3, UTF8::new().actual_size(offset.at(5))?);
        assert_eq!(4, UTF8::new().actual_size(offset.at(8))?);
        assert_eq!(4, UTF8::new().actual_size(offset.at(12))?);
        assert_eq!(2, UTF8::new().actual_size(offset.at(16))?);

        Ok(())
    }

    #[test]
    fn test_to_string() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("A", UTF8::new().to_string(offset.at(0))?);
        assert_eq!("B", UTF8::new().to_string(offset.at(1))?);
        assert_eq!("❄", UTF8::new().to_string(offset.at(2))?);
        assert_eq!("☢", UTF8::new().to_string(offset.at(5))?);
        assert_eq!("𝄞", UTF8::new().to_string(offset.at(8))?);
        assert_eq!("😈", UTF8::new().to_string(offset.at(12))?);
        assert_eq!("÷", UTF8::new().to_string(offset.at(16))?);

        Ok(())
    }

    #[test]
    fn test_too_short() -> SimpleResult<()> {
        let data = b"\xE2".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert!(UTF8::new().to_string(offset.at(0)).is_err());
        assert!(UTF8::new().to_string(offset.at(1)).is_err());

        Ok(())
    }
}
