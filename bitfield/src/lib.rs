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

    pub struct BitSizeMod<const MOD: usize>;

    pub trait AssertMod8 {
        type CheckType;
    }

    impl AssertMod8 for BitSizeMod<0> {
        type CheckType = ZeroMod8;
    }

    impl AssertMod8 for BitSizeMod<1> {
        type CheckType = OneMod8;
    }

    impl AssertMod8 for BitSizeMod<2> {
        type CheckType = TwoMod8;
    }

    impl AssertMod8 for BitSizeMod<3> {
        type CheckType = ThreeMod8;
    }

    impl AssertMod8 for BitSizeMod<4> {
        type CheckType = FourMod8;
    }

    impl AssertMod8 for BitSizeMod<5> {
        type CheckType = FiveMod8;
    }

    impl AssertMod8 for BitSizeMod<6> {
        type CheckType = SixMod8;
    }

    impl AssertMod8 for BitSizeMod<7> {
        type CheckType = SevenMod8;
    }

}

gen_bit_type![1..128];