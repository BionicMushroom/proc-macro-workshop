use derive_builder::Builder;

#[derive(Builder)]
struct Test<T, U, V> {
    field0: T,
    field1: U,
    field2: V,
}

fn main() {}
