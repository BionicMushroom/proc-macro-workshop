#[sorted::check]
fn main() {
    let x = 5;

    #[sorted]
    match x {
        5 => println!("5"),
        1 | 6 => println!("1 or 6"),
        _ => println!("other"),
    }
}
