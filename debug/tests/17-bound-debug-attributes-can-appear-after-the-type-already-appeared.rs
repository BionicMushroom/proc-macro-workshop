#![allow(dead_code)]

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
struct Field<T: Trait> {
    values: Vec<T::Value>,
}

#[derive(CustomDebug)]
struct WrapperStruct<T: Trait> {
    field0: Field<T>,
    field1: u32,

    #[debug(bound = "T::Value: ::core::fmt::Debug")]
    field2: Field<T>,
}

#[derive(CustomDebug)]
enum WrapperEnum<T: Trait> {
    Variant0(Field<T>),
    Variant1(Field<T>, u32),
    Variant2(#[debug(bound = "T::Value: ::core::fmt::Debug")] Field<T>),
}

// Does not implement Debug.
struct NotDebug;

pub trait Trait {
    type Value;
}

impl Trait for NotDebug {
    type Value = u8;
}

fn assert_debug<F: ::core::fmt::Debug>() {}

fn main() {
    assert_debug::<Field<NotDebug>>();
    assert_debug::<WrapperStruct<NotDebug>>();
    assert_debug::<WrapperEnum<NotDebug>>();
}
