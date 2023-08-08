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


// TODO other things

pub trait Specifier {
    const BITS: usize;
    type Type;
}

// bitfield::checks::SevenMod8: bitfield::checks::TotalSizeIsMultipleOfEightBits
pub mod checks {

    pub trait TotalSizeIsMultipleOfEightBits {
        fn test_print2() {
            println!("ok..... TotalSizeIsMultipleOfEightBits");
        }
    }

    pub trait SevenMod8: TotalSizeIsMultipleOfEightBits{
        fn test_print1() {
            println!("ok..... SevenMod8");
        }
    }

}


gen_bit_type![1..128];


const fn _assert_multiple_of_8bits_fn() {
    use crate::checks::TotalSizeIsMultipleOfEightBits;
    if 1 == 6 {
        impl TotalSizeIsMultipleOfEightBits for B8 {}
    } 

    

}
use crate::checks::SevenMod8;
impl SevenMod8 for B8 {}