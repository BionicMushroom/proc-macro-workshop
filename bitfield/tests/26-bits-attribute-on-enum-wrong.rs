#[allow(unused_imports)]
use bitfield::{BitfieldSpecifier, B2};

#[derive(BitfieldSpecifier)]
#[bits = 4]
enum Test {
    Variant0,
    Variant1,
    Variant2,
    Variant3,
}

fn main() {}
