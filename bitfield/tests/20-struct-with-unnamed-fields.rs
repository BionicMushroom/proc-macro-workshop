use bitfield::{bitfield, B16, B8};

#[bitfield]
#[bits = 32]
struct Test(#[bits = 16] B16, B8, B8);

fn main() {
    const VALUE1: u16 = 10;
    const VALUE2: u8 = 20;
    const VALUE3: u8 = 30;

    let mut test = Test::new();

    test.set_0(VALUE1);
    test.set_1(VALUE2);
    test.set_2(VALUE3);

    assert_eq!(test.get_0(), VALUE1);
    assert_eq!(test.get_1(), VALUE2);
    assert_eq!(test.get_2(), VALUE3);
}
