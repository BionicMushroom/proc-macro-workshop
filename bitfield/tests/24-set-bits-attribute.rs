use bitfield::{bitfield, BitfieldSpecifier, B4};

#[derive(BitfieldSpecifier, Debug, PartialEq)]
#[set_bits = 4]
enum SmallPrime {
    Two = 0b0010,
    Three = 0b0011,
    Five = 0b0101,
    Seven = 0b0111,
    Eleven = 0b1011,
    Thirteen = 0b1101,
}

#[bitfield]
struct MyBitfield {
    small_prime0: SmallPrime,
    small_prime1: SmallPrime,
}

fn main() {
    assert_eq!(std::mem::size_of::<SmallPrime>(), 1);
    assert_eq!(std::mem::size_of::<MyBitfield>(), 1);

    let mut bitfield = MyBitfield::new();
    assert_eq!(bitfield.get_small_prime0().unwrap_err().raw_value(), 0);

    bitfield.set_small_prime0(SmallPrime::Seven);
    assert_eq!(bitfield.get_small_prime0().unwrap(), SmallPrime::Seven);
}
