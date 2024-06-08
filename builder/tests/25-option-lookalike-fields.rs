use derive_builder::Builder;

#[derive(Debug)]
struct Option;

mod option {
    #[derive(Debug)]
    pub struct Option<T, U> {
        field0: T,
        field1: U,
    }
}

#[derive(Builder, Debug)]
struct Test {
    option_with_no_generic_arg: Option,
    option_with_multiple_generic_args: option::Option<u32, u32>,
}

fn main() {
    assert_eq!(
        Test::builder().build().unwrap_err().msg(),
        "field `option_with_no_generic_arg` was not set"
    );
    
    assert_eq!(
        Test::builder()
            .option_with_no_generic_arg(Option)
            .build()
            .unwrap_err()
            .msg(),
        "field `option_with_multiple_generic_args` was not set"
    );
}
