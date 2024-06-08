use derive_builder::Builder;

#[derive(Builder)]
struct Test {
    #[builder = "whatever"]
    field: u32,
}

fn main() {}
