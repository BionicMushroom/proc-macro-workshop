#[sorted::check]
fn main() {
    let x = 10;
    let y = 5;

    #[sorted]
    match x {
        0 if y == 5 => println!("first"),
        _ => println!("second"),
    }
}
