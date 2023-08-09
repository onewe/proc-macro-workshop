// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.
pub use bitfield_impl::bitfield;
use bitfield_impl::gen_bit_type;
use checks::TotalSizeIsMultipleOfEightBits;

use crate::checks::{SevenMod8, TwoMod8};

// TODO other things

pub trait Specifier {
    const BITS: usize;
    type Type;
}

pub trait StructBits {
    const TOTAL_BITS: usize;
}

pub mod checks {
    pub trait TotalSizeIsMultipleOfEightBits{}

    impl TotalSizeIsMultipleOfEightBits for ZeroMod8{}

    pub struct ZeroMod8;

    pub struct  OneMod8;
    pub struct TwoMod8;
    pub struct ThreeMod8;
    pub struct FourMod8;
    pub struct FiveMod8;
    pub struct SixMod8;
    pub struct SevenMod8;

}


gen_bit_type![1..128];




// #[macro_export]
// macro_rules! check_mod8 {
//     ($mod:expr) => {
//         match $mod {
//             0 => {
//                 struct _AssertMod8 where bitfield::checks::ZeroMod8: TotalSizeIsMultipleOfEightBits;
//             },
//             1 => {
//                 struct _AssertMod8 where bitfield::checks::OneMod8: TotalSizeIsMultipleOfEightBits;
//             },
//             2 => {
//                 struct _AssertMod8 where bitfield::checks::TwoMod8: TotalSizeIsMultipleOfEightBits;
//             },
//             3 => {
//                 struct _AssertMod8 where bitfield::checks::ThreeMod8: TotalSizeIsMultipleOfEightBits;
//             },
//             4 => {
//                 struct _AssertMod8 where bitfield::checks::FourMod8: TotalSizeIsMultipleOfEightBits;
//             },
//             5 => {
//                 struct _AssertMod8 where bitfield::checks::FiveMod8: TotalSizeIsMultipleOfEightBits;
//             },
//             6 => {
//                 struct _AssertMod8 where bitfield::checks::SixMod8: TotalSizeIsMultipleOfEightBits;
//             },
//             7 => {
//                 struct _AssertMod8 where bitfield::checks::SevenMod8: TotalSizeIsMultipleOfEightBits;
//             }
//         }
//     };
// }


const MAX_DD: usize = 10;

const _: () = {

    if MAX_DD > 20 {
        impl TotalSizeIsMultipleOfEightBits for  SevenMod8{}
    } else {
        impl TotalSizeIsMultipleOfEightBits for  TwoMod8{}
    }

};