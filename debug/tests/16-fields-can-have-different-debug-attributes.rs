use derive_debug::CustomDebug;

#[derive(CustomDebug)]
struct Test0<T: ::core::fmt::Binary> {
    #[debug = "0b{:08b}"]
    #[debug(bound = "T: ::core::fmt::Debug")]
    field0: T,
}

#[derive(CustomDebug)]
struct Test1<T: ::core::fmt::Binary> {
    #[debug(bound = "T: ::core::fmt::Debug")]
    #[debug = "0b{:08b}"]
    field0: T,
}

#[derive(CustomDebug)]
enum Test2<T: ::core::fmt::Binary> {
    Variant0(
        #[debug = "0b{:08b}"]
        #[debug(bound = "T: ::core::fmt::Debug")]
        T,
    ),
}

#[derive(CustomDebug)]
enum Test3<T: ::core::fmt::Binary> {
    Variant0(
        #[debug(bound = "T: ::core::fmt::Debug")]
        #[debug = "0b{:08b}"]
        T,
    ),
}

fn main() {
    assert!(format!("{:?}", Test0 { field0: 0 }) == "Test0 { field0: 0b00000000 }");
    assert!(format!("{:?}", Test1 { field0: 0 }) == "Test1 { field0: 0b00000000 }");
    assert!(format!("{:?}", Test2::Variant0(0)) == "Variant0(0b00000000)");
    assert!(format!("{:?}", Test3::Variant0(0)) == "Variant0(0b00000000)");
}
