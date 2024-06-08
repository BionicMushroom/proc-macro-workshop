use derive_builder::Builder;

#[derive(Builder)]
union Union {
    field0: u32,
    field1: u64,
}

fn main() {}
