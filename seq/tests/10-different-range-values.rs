use seq::seq;

seq!(N in 340282366920938463463374607431768211455u128..=340282366920938463463374607431768211455u128 {
    struct Test~N;
});

seq!(N in 5u8..=5u8 {
    struct Test~N;
});

fn main() {
    let _ = Test340282366920938463463374607431768211455;
    let _ = Test5;
}
