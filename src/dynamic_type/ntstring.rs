use simple_error::{bail, SimpleResult};

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use sized_number::{Endian, Context};
use crate::{H2Type, H2Types, H2TypeTrait, Offset};
use crate::alignment::Alignment;
use crate::basic_type::{Character, CharacterType};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct NTString {
    character: H2Type,
}

// impl From<NTString> for DynamicType {
//     fn from(o: NTString) -> DynamicType {
//         DynamicType::from(DynamicType::NTString(o))
//     }
// }

impl NTString {
    pub fn new(character: H2Type) -> Self {
        Self {
            character: character,
        }
    }

    fn analyze(&self, offset: Offset) -> SimpleResult<(u64, Vec<char>)> {
        let mut position = offset.position();
        let mut result = Vec::new();

        loop {
            let this_offset = offset.at(position);
            let this_size = self.character.actual_size(this_offset)?;
            let this_character = self.character.to_char(this_offset)?;

            result.push(this_character);
            position = position + this_size;

            if this_character == '\0' {
                break;
            }
        }

        Ok((position, result))
        //bail!("TODO");
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
        Ok(self.analyze(offset)?.1.iter().collect())
    }

    fn children(&self, _offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        bail!("TODO");
        // let size = self.size(context)?;
        // let mut result: Vec<ResolvedType> = Vec::new();

        // TODO
        // if size > 1 {
        //     result.push(ResolvedType::new(context.position(), None,
        //         StaticType::from(H2Array::new(
        //             size - 1,
        //             StaticType::from(Character::new()),
        //         )),
        //     ));
        // }

        // result.push(ResolvedType::new(context.position() + size - 1, None,
        //     StaticType::from(H2Number::new(SizedDefinition::U8, SizedDisplay::Decimal))
        // ));


        //Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    #[test]
    fn test_utf8_string() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7\x00".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let a = NTString::new(Character::new(CharacterType::UTF8));
        assert_eq!("AB❄☢𝄞😈÷\0", a.to_string(offset)?);

        Ok(())
    }
}
