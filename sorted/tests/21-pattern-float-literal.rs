#[sorted::check]
fn main() {
    let x = 0.5;

    #[sorted]
    match x {
        0.5 => println!("0.5"),
        0.1 => println!("0.1"),
        _ => println!("other"),
    }
}
