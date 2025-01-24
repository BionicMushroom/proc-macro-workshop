use derive_debug::CustomDebug;

#[derive(CustomDebug)]
#[debug]
enum Test0 {}

#[derive(CustomDebug)]
#[debug = "0b{:08b}"]
enum Test1 {}

#[derive(CustomDebug)]
#[debug(bnd = "T: ::core::fmt::Debug")]
enum Test2<T> {
    Variant0(T),
}

#[derive(CustomDebug)]
#[debug(bound = "T: ::core::fmt::Debug")]
#[debug(bound = "T: ::core::fmt::Debug")]
enum Test3<T> {
    Variant0(T),
}

#[derive(CustomDebug)]
enum Test4 {
    Variant0(
        #[debug = "0b{:08b}"]
        #[debug = "0b{:08b}"]
        u8,
    ),
}

#[derive(CustomDebug)]
enum Test5<T> {
    Variant0(
        #[debug(bound = "T: ::core::fmt::Debug")]
        #[debug(bound = "T: ::core::fmt::Debug")]
        T,
    ),
}

#[derive(CustomDebug)]
enum Test6<T> {
    Variant0(
        #[debug(bound = "T: ::core::fmt::Debug")]
        #[debug = "0b{:08b}"]
        #[debug(bound = "T: ::core::fmt::Debug")]
        T,
    ),
}

#[derive(CustomDebug)]
#[debug(bound = "T: ::core::fmt::Debug")]
enum Test7<T> {
    Variant0(#[debug(bound = "T: ::core::fmt::Debug")] T),
}

#[derive(CustomDebug)]
enum Test8<T> {
    Variant0(#[debug(bound = "T: ::core::fmt::Debug")] T),
    Variant1(#[debug(bound = "T: ::core::fmt::Debug")] T),
}

#[derive(CustomDebug)]
enum Test9<T> {
    Variant0(
        #[debug(bound = "T: ::core::fmt::Debug")] T,
        #[debug(bound = "T: ::core::fmt::Debug")] T,
    ),
}

#[derive(CustomDebug)]
enum Test10<T> {
    Variant0(#[debug] T),
}

#[derive(CustomDebug)]
#[debug(bound = "T: ::core::fmt::Debug")]
enum Test11<T> {
    Variant0(#[debug] T),
}

#[derive(CustomDebug)]
enum Test12<T> {
    #[debug(bound = "T: ::core::fmt::Debug")]
    Variant0(T),
}

#[derive(CustomDebug)]
enum Test13<T> {
    #[debug = "0b{:08b}"]
    Variant0(T),
}

#[derive(CustomDebug)]
enum Test14<T> {
    #[debug]
    Variant0(T),
}

fn main() {}
