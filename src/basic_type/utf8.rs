use simple_error::{SimpleResult, bail};
use std::char;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use sized_number::{Endian, Context};

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

    fn try_utf8(&self, slice: &[u8]) -> SimpleResult<char> {
      if let Ok(s) = std::str::from_utf8(&slice) {
          if let Some(c) = s.chars().next() {
              return Ok(c);
          }
      }

      bail!("Could not convert");
    }

    fn read_utf8(&self, offset: Offset) -> SimpleResult<(u64, char)> {
        let context = offset.get_dynamic()?;
        let slice = context.as_slice();

        for i in 1..=4 {
            if slice.len() >= i {
                if let Ok(c) = self.try_utf8(&slice[0..i]) {
                    return Ok((i as u64, c));
                }
            }
        }

        bail!("Could not decode utf8 string!");
    }
}

impl H2TypeTrait for UTF8 {
    fn is_static(&self) -> bool {
        false
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        let (size, _) = self.read_utf8(offset)?;

        Ok(size)
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        let (_, c) = self.read_utf8(offset)?;

        Ok(format!("{}", c))
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
