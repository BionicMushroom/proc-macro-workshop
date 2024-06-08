use derive_builder::Builder;

mod test {
    #[derive(super::Builder, Debug)]
    pub struct Test {
        field: u32,
    }
}

fn main() {
    let _ = test::Test::builder().field(0).build().unwrap();
    
    assert_eq!(
        test::Test::builder().build().unwrap_err().msg(),
        "field `field` was not set"
    );
}
