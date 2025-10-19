#[allow(unused_imports)]
use bitfield::{BitfieldSpecifier, B2};

#[derive(BitfieldSpecifier)]
#[bits = 2]
enum Test {
    Variant0,
    Variant1,
    Variant2,
    Variant3(u8),
}

fn main() {}
