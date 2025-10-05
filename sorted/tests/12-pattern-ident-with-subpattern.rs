#[sorted::check]
fn main() {
    let x = 10;

    #[sorted]
    match x {
        ref num @ 1 => println!("{num}"),
        0 => println!("0"),
        _ => println!("everything else"),
    }
}
