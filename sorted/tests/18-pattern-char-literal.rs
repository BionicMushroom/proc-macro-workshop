#[sorted::check]
fn main() {
    let x = 'a';

    #[sorted]
    match x {
        'd' => println!("d"),
        'a' => println!("a"),
        _ => println!("other"),
    }
}
