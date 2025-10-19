#[allow(unused_imports)]
use bitfield::{bitfield, B4, B8};

#[bitfield]
#[bits = 18446744073709551616]
struct Test {
    field0: B8,
    field1: B4,
    field2: B4,
}

fn main() {}
