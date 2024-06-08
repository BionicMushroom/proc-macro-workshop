use derive_builder::Builder;

struct Vec<T, U> {
    field0: T,
    field1: U,
}

#[derive(Builder)]
struct Test {
    #[builder(each = "whatever")]
    field: Vec<u32, u32>,
}

fn main() {}
