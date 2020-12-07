mod alignment;
pub use alignment::Alignment;

mod resolved_type;
pub use resolved_type::ResolvedType;

mod offset;
pub use offset::Offset;

mod h2types;
pub use h2types::H2Types;

mod h2typetrait;
pub use h2typetrait::H2TypeTrait;

mod h2type;
pub use h2type::H2Type;

pub mod basic_type;
pub mod complex_type;
// pub mod dynamic_type;

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use simple_error::SimpleResult;
//     use sized_number::Context;
//     use basic_type::Character;

//     #[test]
//     fn test_character() -> SimpleResult<()> {
//         let t = Character::new();
//         let data = b"ABCD".to_vec();
//         let s_offset = Offset::Static(0);
//         let d_offset = Offset::Dynamic(Context::new(&data));

//         assert_eq!(1, t.actual_size(s_offset)?);
//         assert_eq!(1, t.actual_size(d_offset)?);

//         assert_eq!("A", t.to_string(d_offset.at(0))?);
//         assert_eq!("B", t.to_string(d_offset.at(1))?);
//         assert_eq!("C", t.to_string(d_offset.at(2))?);
//         assert_eq!("D", t.to_string(d_offset.at(3))?);

//         assert_eq!(0, t.resolve_partial(s_offset)?.len());
//         assert_eq!(0, t.resolve_partial(d_offset)?.len());

//         let resolved = t.resolve_full(s_offset)?;
//         assert_eq!(1, resolved.len());
//         assert_eq!(0..1, resolved[0].actual_range);
//         assert_eq!("Character", resolved[0].to_string());

//         let resolved = t.resolve_full(s_offset.at(1))?;
//         assert_eq!(1, resolved.len());
//         assert_eq!(1..2, resolved[0].actual_range);
//         assert_eq!("Character", resolved[0].to_string());

//         let resolved = t.resolve_full(d_offset)?;
//         assert_eq!(1, resolved.len());
//         assert_eq!(0..1, resolved[0].actual_range);
//         assert_eq!("A", resolved[0].to_string());

//         let resolved = t.resolve_full(d_offset.at(1))?;
//         assert_eq!(1, resolved.len());
//         assert_eq!(1..2, resolved[0].actual_range);
//         assert_eq!("B", resolved[0].to_string());

//         Ok(())
//     }

//     // #[test]
//     // fn test_align() -> SimpleResult<()> {
//     //     // Align to 4-byte boundaries
//     //     let t = H2Type::from((4, Character::new()));
//     //     let data = b"ABCD".to_vec();
//     //     let context = Context::new(&data);

//     //     assert_eq!(1, t.size()?);
//     //     assert_eq!(1, t.size(Context::new(&data).at(0))?);
//     //     assert_eq!("A", t.to_string(Context::new(&data).at(0))?);
//     //     assert_eq!("B", t.to_string(Context::new(&data).at(1))?);
//     //     assert_eq!("C", t.to_string(Context::new(&data).at(2))?);
//     //     assert_eq!("D", t.to_string(Context::new(&data).at(3))?);

//     //     assert_eq!(0, t.children_static(0)?.len());
//     //     assert_eq!(0, t.resolve_partial(Context::new(&data).at(0))?.len());

//     //     let resolved = t.resolve(Context::new(&data).at(0))?;
//     //     assert_eq!(1, resolved.len());
//     //     assert_eq!(0..1, resolved[0].offset);
//     //     assert_eq!("A", resolved[0].to_string(Context::new(&data))?);

//     //     let resolved = t.resolve(Context::new(&data).at(1))?;
//     //     assert_eq!(1, resolved.len());
//     //     assert_eq!(1..2, resolved[0].offset);
//     //     assert_eq!("B", resolved[0].to_string(Context::new(&data))?);

//     //     Ok(())
//     // }

//     #[test]
//     fn test_padding() -> SimpleResult<()> {
//         Ok(())
//     }

//     #[test]
//     fn test_pointer() -> SimpleResult<()> {
//         Ok(())
//     }

//     #[test]
//     fn test_static_array() -> SimpleResult<()> {
//         Ok(())
//     }

//     #[test]
//     fn test_dynamic_array() -> SimpleResult<()> {
//         Ok(())
//     }

//     #[test]
//     fn test_aligned_array() -> SimpleResult<()> {
//         Ok(())
//     }

//     #[test]
//     fn test_static_struct() -> SimpleResult<()> {
//         Ok(())
//     }

//     #[test]
//     fn test_dynamic_struct() -> SimpleResult<()> {
//         Ok(())
//     }

//     #[test]
//     fn test_enum() -> SimpleResult<()> {
//         Ok(())
//     }

//     #[test]
//     fn test_ntstring() -> SimpleResult<()> {
//         Ok(())
//     }
// }
