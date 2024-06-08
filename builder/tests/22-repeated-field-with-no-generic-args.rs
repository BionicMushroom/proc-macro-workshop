use derive_builder::Builder;

struct Vec;

#[derive(Builder)]
struct Test {
    #[builder(each = "whatever")]
    field: Vec,
}

fn main() {}
