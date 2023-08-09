use bitfield::{B8, B1};
use bitfield::checks::TotalSizeIsMultipleOfEightBits;


const MAX_LEN: u8  = 1;

const MAX_LEN_MOD: u8 = 1 % 8;


#[test]
fn tests() {
    let t = trybuild::TestCases::new();

    let a: Box<dyn bitfield::checks::TotalSizeIsMultipleOfEightBits> = Box::new(bitfield::checks::ThreeMod8);
    // t.pass("tests/01-specifier-types.rs");
    // t.pass("tests/02-storage.rs");
    // t.pass("tests/03-accessors.rs");
    // t.compile_fail("tests/04-multiple-of-8bits.rs");
    //t.pass("tests/05-accessor-signatures.rs");
    //t.pass("tests/06-enums.rs");
    //t.pass("tests/07-optional-discriminant.rs");
    //t.compile_fail("tests/08-non-power-of-two.rs");
    //t.compile_fail("tests/09-variant-out-of-range.rs");
    //t.pass("tests/10-bits-attribute.rs");
    //t.compile_fail("tests/11-bits-attribute-wrong.rs");
    //t.pass("tests/12-accessors-edge.rs");
}



