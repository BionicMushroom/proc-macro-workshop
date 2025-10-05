#[sorted::check]
fn main() {
    let x = "abc";

    #[sorted]
    match x {
        "def" => println!("def"),
        "abc def" => println!("abc def"),
        _ => println!("other"),
    }
}
