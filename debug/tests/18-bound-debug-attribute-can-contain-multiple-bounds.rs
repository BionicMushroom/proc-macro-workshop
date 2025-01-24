use derive_debug::CustomDebug;

#[derive(CustomDebug)]
struct Test0<T> {
    #[debug(bound = "T: ::core::fmt::Debug + ::core::fmt::Binary")]
    #[debug = "0b{:08b}"]
    field0: T,
}

#[derive(CustomDebug)]
#[debug(
    bound = "T: ::core::fmt::Debug + ::core::fmt::Binary, U: ::core::fmt::Debug + ::core::fmt::Binary"
)]
struct Test1<T, U> {
    #[debug = "0b{:08b}"]
    field0: T,
    #[debug = "0b{:08b}"]
    field1: U,
}

#[derive(CustomDebug)]
enum Test2<T> {
    Variant0(
        #[debug(bound = "T: ::core::fmt::Debug + ::core::fmt::Binary")]
        #[debug = "0b{:08b}"]
        T,
    ),
}

#[derive(CustomDebug)]
#[debug(
    bound = "T: ::core::fmt::Debug + ::core::fmt::Binary, U: ::core::fmt::Debug + ::core::fmt::Binary"
)]
enum Test3<T, U> {
    Variant0(#[debug = "0b{:08b}"] T, #[debug = "0b{:08b}"] U),
}

fn main() {
    assert!(format!("{:?}", Test0 { field0: 0 }) == "Test0 { field0: 0b00000000 }");
    
    assert!(
        format!(
            "{:?}",
            Test1 {
                field0: 0,
                field1: 0
            }
        ) == "Test1 { field0: 0b00000000, field1: 0b00000000 }"
    );
    
    assert!(format!("{:?}", Test2::Variant0(0)) == "Variant0(0b00000000)");
    assert!(format!("{:?}", Test3::Variant0(0, 0)) == "Variant0(0b00000000, 0b00000000)");
}
