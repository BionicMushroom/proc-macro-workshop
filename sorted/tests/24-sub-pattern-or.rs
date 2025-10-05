#[sorted::check]
fn main() {
    let x = 5;

    #[sorted]
    match x {
        5 => println!("5"),
        7 | 8 | 6 => println!("6, 7 or 8"),
        _ => println!("other"),
    }
}
