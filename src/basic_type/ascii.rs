use simple_error::{bail, SimpleResult};

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use crate::{H2Type, H2Types, H2TypeTrait, Offset};
use crate::alignment::Alignment;

/// Configure whether invalid ASCII characters are an error or just replaced
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum StrictASCII {
    Strict,
    Permissive,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ASCII {
    strict: StrictASCII,
}

// TODO: Add strict / loose options
impl ASCII {
    pub fn new_aligned(alignment: Alignment, strict: StrictASCII) -> H2Type {
        H2Type::new(alignment, H2Types::ASCII(Self {
            strict: strict,
        }))
    }

    pub fn new(strict: StrictASCII) -> H2Type {
        Self::new_aligned(Alignment::None, strict)
    }
}

impl H2TypeTrait for ASCII {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(1)
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("ASCII".to_string()),
            Offset::Dynamic(context) => {
                let number = context.read_u8()?;

                match number > 0x1F && number < 0x7F {
                    true  => Ok(format!("'{}'", (number as char))),
                    false => match self.strict {
                        StrictASCII::Strict     => bail!("Invalid ASCII character: {:#x}", number),
                        StrictASCII::Permissive => Ok("'�'".to_string()),
                    },
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
    fn test_ascii_type_unaligned() -> SimpleResult<()> {
        let c = ASCII::new(StrictASCII::Permissive);

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

        let r = ASCII::new(StrictASCII::Permissive).resolve(offset)?;
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
        let c = ASCII::new_aligned(Alignment::Loose(4), StrictASCII::Permissive);

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

        let r = ASCII::new_aligned(Alignment::Loose(4), StrictASCII::Permissive).resolve(offset)?;
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
        let data = b"\x00\x1F\x20\x41\x42\x7e\x7f\x80\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));
        let t = ASCII::new(StrictASCII::Permissive);

        assert_eq!("'�'", t.to_string(offset.at(0))?);
        assert_eq!("'�'", t.to_string(offset.at(1))?);
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
        let data = b"\x00\x1F\x20\x41\x42\x7e\x7f\x80\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));
        let t = ASCII::new(StrictASCII::Strict);

        assert!(t.to_string(offset.at(0)).is_err());
        assert!(t.to_string(offset.at(1)).is_err());
        assert!(t.to_string(offset.at(6)).is_err());
        assert!(t.to_string(offset.at(7)).is_err());
        assert!(t.to_string(offset.at(8)).is_err());

        assert_eq!("' '", t.to_string(offset.at(2))?);
        assert_eq!("'A'", t.to_string(offset.at(3))?);
        assert_eq!("'B'", t.to_string(offset.at(4))?);
        assert_eq!("'~'", t.to_string(offset.at(5))?);

        Ok(())
    }
}
