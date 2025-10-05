use sorted::sorted;

#[sorted]
enum Test {
    A,
    B,
    C,
}

#[sorted::check]
fn main() {
    let x = Test::A;

    #[sorted]
    match x {
        Test::B => println!("B"),
        Test::A => println!("A"),
        Test::C => println!("C"),
    }
}
