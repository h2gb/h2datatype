use simple_error::SimpleResult;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2Types, H2TypeTrait, Offset};
use crate::alignment::Alignment;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Character {
}

impl Character {
    pub fn new_aligned(alignment: Alignment) -> H2Type {
        H2Type::new(alignment, H2Types::Character(Self {
        }))
    }

    pub fn new() -> H2Type {
        Self::new_aligned(Alignment::None)
    }
}

impl H2TypeTrait for Character {
    fn is_static(&self) -> bool {
        true
    }

    fn size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(1)
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("Character".to_string()),
            Offset::Dynamic(context) => {
                let number = context.read_u8()?;

                match number > 0x1F && number < 0x7F {
                    true  => Ok((number as char).to_string()),
                    false => Ok("<invalid>".to_string()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    #[test]
    fn test_character() -> SimpleResult<()> {
        let data = b"\x00\x1F\x20\x41\x42\x7e\x7f\x80\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("<invalid>", Character::new().to_string(offset.at(0))?);
        assert_eq!("<invalid>", Character::new().to_string(offset.at(1))?);
        assert_eq!(" ",         Character::new().to_string(offset.at(2))?);
        assert_eq!("A",         Character::new().to_string(offset.at(3))?);
        assert_eq!("B",         Character::new().to_string(offset.at(4))?);
        assert_eq!("~",         Character::new().to_string(offset.at(5))?);
        assert_eq!("<invalid>", Character::new().to_string(offset.at(6))?);
        assert_eq!("<invalid>", Character::new().to_string(offset.at(7))?);
        assert_eq!("<invalid>", Character::new().to_string(offset.at(8))?);

        Ok(())
    }
}
