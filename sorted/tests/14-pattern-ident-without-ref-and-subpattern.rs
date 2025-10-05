#[allow(unreachable_patterns)]
#[sorted::check]
fn main() {
    let x = 10;

    #[sorted]
    match x {
        def => println!("{def}"),
        abc => println!("{abc}"),
    }
}
