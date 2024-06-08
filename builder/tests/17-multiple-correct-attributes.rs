use derive_builder::Builder;

#[derive(Builder)]
struct Test {
    #[builder(each = "first")]
    #[builder(each = "second")]
    field: u32,
}

fn main() {}
