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


// macro_rules!  gen_b_type{
//     ($($id:literal),*) => {
//         paste! {
//             $(
//                 pub struct [<B $id>] {

//                 }

//                 impl $crate::Specifier for [<B $id>] {
//                     const BITS: usize = $id;
//                 }
//             )*
           
//         }
//     };
// }

// gen_b_type![
//     1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,
//     31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,
//     58,59,60,61,62,63,64
// ];


// impl Specifier for bool {
//     const BITS: usize = 1;
// }

gen_bit_type![1..2];