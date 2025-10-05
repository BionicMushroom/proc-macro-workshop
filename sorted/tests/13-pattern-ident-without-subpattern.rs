#[sorted::check]
fn main() {
    let x = 10;

    #[sorted]
    match x {
        0 => println!("0"),
        ref num => println!("{num}"),
    }
}
