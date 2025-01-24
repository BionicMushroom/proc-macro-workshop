use derive_debug::CustomDebug;

#[derive(CustomDebug)]
#[debug]
struct Test0;

#[derive(CustomDebug)]
#[debug = "0b{:08b}"]
struct Test1;

#[derive(CustomDebug)]
#[debug(bnd = "T: ::core::fmt::Debug")]
struct Test2<T> {
    field0: T,
}

#[derive(CustomDebug)]
#[debug(bound = "T: ::core::fmt::Debug")]
#[debug(bound = "T: ::core::fmt::Debug")]
struct Test3<T> {
    field0: T,
}

#[derive(CustomDebug)]
struct Test4 {
    #[debug = "0b{:08b}"]
    #[debug = "0b{:08b}"]
    field0: u8,
}

#[derive(CustomDebug)]
struct Test5<T> {
    #[debug(bound = "T: ::core::fmt::Debug")]
    #[debug(bound = "T: ::core::fmt::Debug")]
    field0: T,
}

#[derive(CustomDebug)]
struct Test6<T> {
    #[debug(bound = "T: ::core::fmt::Debug")]
    #[debug = "0b{:08b}"]
    #[debug(bound = "T: ::core::fmt::Debug")]
    field0: T,
}

#[derive(CustomDebug)]
#[debug(bound = "T: ::core::fmt::Debug")]
struct Test7<T> {
    #[debug(bound = "T: ::core::fmt::Debug")]
    field0: T,
}

#[derive(CustomDebug)]
struct Test8<T> {
    #[debug(bound = "T: ::core::fmt::Debug")]
    field0: T,

    #[debug(bound = "T: ::core::fmt::Debug")]
    field1: T,
}

#[derive(CustomDebug)]
struct Test9<T> {
    #[debug]
    field0: T,
}

#[derive(CustomDebug)]
#[debug(bound = "T: ::core::fmt::Debug")]
struct Test10<T> {
    #[debug]
    field0: T,
}

fn main() {}
