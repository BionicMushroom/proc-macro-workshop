#[allow(unused_imports)]
use bitfield::{BitfieldSpecifier, B2};

#[allow(clippy::duplicated_attributes)]
#[derive(BitfieldSpecifier)]
#[set_bits = 2]
#[set_bits = 2]
enum Test {
    Variant0,
    Variant1,
    Variant2,
    Variant3,
}

fn main() {}
