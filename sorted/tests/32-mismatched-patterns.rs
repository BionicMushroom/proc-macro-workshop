#[sorted::check]
fn main() {
    const X: u32 = 0;
    let x = 0;

    #[sorted]
    match x {
        X => println!("0"),
        1 => println!("1"),
        _ => println!("other"),
    }
}
