use derive_builder::Builder;

#[derive(Builder)]
struct Test {
    #[builder(each = "whatever")]
    field: std::result::Result<u32, u32>,
}

fn main() {}
