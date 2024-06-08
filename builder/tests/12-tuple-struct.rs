use derive_builder::Builder;

#[derive(Builder)]
struct Test(i32, u32, String, Option<String>, Vec<String>);

fn main() {}
