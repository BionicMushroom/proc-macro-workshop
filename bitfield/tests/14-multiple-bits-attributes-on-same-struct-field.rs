#[allow(unused_imports)]
use bitfield::{bitfield, B8};

#[bitfield]
struct Test {
    #[bits = 8]
    #[bits = 8]
    field: B8,
}

fn main() {}
