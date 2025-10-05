#[allow(clippy::match_bool)]
#[sorted::check]
fn main() {
    let x = false;

    #[sorted]
    match x {
        true => println!("true"),
        false => println!("false"),
    }
}
