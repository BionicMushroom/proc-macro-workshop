use bitfield::{BitfieldSpecifier, B2};

#[derive(BitfieldSpecifier)]
#[set_bits = 2]
#[bits = 2]
enum Test {
    Variant0,
    Variant1,
    Variant2,
    Variant3,
}

fn main() {}
