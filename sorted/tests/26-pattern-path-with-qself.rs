trait TestTrait {
    const VALUE: u32;
}

struct TestStruct;

impl TestTrait for TestStruct {
    const VALUE: u32 = 0;
}

#[sorted::check]
fn main() {
    let x = 0;

    #[sorted]
    match x {
        <TestStruct as TestTrait>::VALUE => println!("first"),
        _ => println!("other"),
    }
}
