use bitfield::{bitfield, B8};

#[bitfield]
struct Test {
    r#type: B8,
}

fn main() {
    const VALUE: u8 = 10;
    let mut test = Test::new();

    test.set_type(VALUE);
    assert_eq!(test.get_type(), VALUE);
}
