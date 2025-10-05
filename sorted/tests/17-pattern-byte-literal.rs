#[sorted::check]
fn main() {
    let x = b'a';

    #[sorted]
    match x {
        b'd' => println!("d"),
        b'a' => println!("a"),
        _ => println!("other"),
    }
}
