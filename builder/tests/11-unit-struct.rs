use derive_builder::Builder;

#[derive(Builder)]
struct UnitStruct;

fn main() {
    let _ = UnitStruct::builder().build().unwrap();
}
