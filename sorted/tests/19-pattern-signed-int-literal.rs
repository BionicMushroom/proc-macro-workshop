#[sorted::check]
fn main() {
    let x = 15i128;

    #[sorted]
    match x {
        0 => println!("0"),
        1 => println!("1"),
        -170141183460469231731687303715884105728 => {
            println!("-170141183460469231731687303715884105728")
        }
        _ => println!("other"),
    }
}
