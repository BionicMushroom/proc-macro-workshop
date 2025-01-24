#![allow(dead_code)]

use core::{fmt::Debug, marker::PhantomData};

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
enum Test0 {}

#[derive(CustomDebug)]
enum Test1 {
    Variant0(u8, u8),
    Variant1,
}

#[derive(CustomDebug)]
enum Test2 {
    Variant0(u8, #[debug = "0b{:08b}"] u8),
}

#[derive(CustomDebug)]
enum Test3<T> {
    Variant0(T),
}

#[derive(CustomDebug)]
enum Test4<T> {
    Variant0(Vec<PhantomData<T>>),
}

#[derive(CustomDebug)]
pub enum Test5<T: Trait> {
    Variant0(Vec<T::Value>),
}

#[derive(CustomDebug)]
#[debug(bound = "T::Value: Debug")]
pub enum Test6<T: Trait> {
    Variant0(Test5<T>),
}

#[derive(CustomDebug)]
pub enum Test7<T: Trait, U> {
    Variant0(#[debug(bound = "T::Value: Debug")] Test5<T>, U),
}

// Does not implement Debug.
struct NotDebug;

pub trait Trait {
    type Value;
}

impl Trait for NotDebug {
    type Value = u8;
}

impl Trait for u8 {
    type Value = u8;
}

fn assert_debug<F: Debug>() {}

fn main() {
    assert!(format!("{:?}", Test1::Variant0(0, 0)) == "Variant0(0, 0)");
    assert!(format!("{:?}", Test1::Variant1) == "Variant1");
    assert!(format!("{:?}", Test2::Variant0(0, 0)) == "Variant0(0, 0b00000000)");
    assert!(format!("{:?}", Test3::Variant0(0)) == "Variant0(0)");

    assert_debug::<Test0>();
    assert_debug::<Test4<NotDebug>>();
    assert_debug::<Test5<NotDebug>>();
    assert_debug::<Test6<NotDebug>>();
    assert_debug::<Test7<NotDebug, u8>>();
}
