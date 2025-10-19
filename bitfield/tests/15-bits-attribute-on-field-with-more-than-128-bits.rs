#[allow(unused_imports)]
use bitfield::{bitfield, B8};

#[bitfield]
struct Test {
    #[bits = 129]
    field: B8,
}

fn main() {}
