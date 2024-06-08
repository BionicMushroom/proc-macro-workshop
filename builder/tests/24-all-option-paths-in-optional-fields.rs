use derive_builder::Builder;
use std::option;

#[derive(Builder)]
struct Test {
    field0: Option<u32>,
    field1: option::Option<u32>,
    field2: std::option::Option<u32>,
    field3: ::std::option::Option<u32>,
}

fn main() {
    let _ = Test::builder().build().unwrap();
}
