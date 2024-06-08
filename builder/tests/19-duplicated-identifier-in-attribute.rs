use derive_builder::Builder;

#[derive(Builder)]
struct Test {
    #[builder(each = "add_field")]
    field0: Vec<u32>,

    #[builder(each = "add_field")]
    field1: Vec<u32>,
}

fn main() {}
