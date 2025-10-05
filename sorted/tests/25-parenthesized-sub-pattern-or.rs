#[allow(clippy::unnested_or_patterns, unused_parens)]
#[sorted::check]
fn main() {
    let x = 5;

    #[sorted]
    match x {
        5 => println!("5"),
        (8 | 9) | (6 | 7) => println!("6, 7, 8 or 9"),
        _ => println!("other"),
    }
}
