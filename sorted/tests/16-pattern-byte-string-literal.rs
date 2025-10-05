#[sorted::check]
fn main() {
    let x = b"abc abc";

    #[sorted]
    match x {
        b"def abc" => println!("def abc"),
        b"abc def" => println!("abc def"),
        _ => println!("other"),
    }
}
