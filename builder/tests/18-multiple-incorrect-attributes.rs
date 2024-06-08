use derive_builder::Builder;

#[derive(Builder)]
struct Test {
    #[builder = "first"]
    #[builder = "second"]
    field: u32,
}

fn main() {}
