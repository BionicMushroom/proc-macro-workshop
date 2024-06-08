use derive_builder::Builder;

#[derive(Builder)]
struct Test {
    #[builder]
    field: u32,
}

fn main() {}
