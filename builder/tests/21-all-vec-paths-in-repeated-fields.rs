use derive_builder::Builder;
use std::vec;

#[derive(Builder)]
struct Test {
    #[builder(each = "push_field0")]
    field0: Vec<u32>,

    #[builder(each = "push_field1")]
    field1: vec::Vec<u32>,

    #[builder(each = "push_field2")]
    field2: std::vec::Vec<u32>,

    #[builder(each = "push_field3")]
    field3: ::std::vec::Vec<u32>,
}

fn main() {
    let t = Test::builder()
        .push_field0(0)
        .push_field0(1)
        .push_field1(2)
        .push_field1(3)
        .push_field2(4)
        .push_field2(5)
        .push_field3(6)
        .push_field3(7)
        .build()
        .unwrap();

    assert_eq!(t.field0, vec![0, 1]);
    assert_eq!(t.field1, vec![2, 3]);
    assert_eq!(t.field2, vec![4, 5]);
    assert_eq!(t.field3, vec![6, 7]);
}
