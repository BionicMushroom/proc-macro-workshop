use core::{fmt::Debug, marker::PhantomData};

use derive_debug::CustomDebug;

#[derive(CustomDebug)]
struct Test0();

#[derive(CustomDebug)]
struct Test1(u8, u8);

#[derive(CustomDebug)]
struct Test2(u8, #[debug = "0b{:08b}"] u8);

#[derive(CustomDebug)]
struct Test3<T>(T);

#[derive(CustomDebug)]
struct Test4<T>(Vec<PhantomData<T>>);

#[derive(CustomDebug)]
pub struct Test5<T: Trait>(Vec<T::Value>);

#[derive(CustomDebug)]
#[debug(bound = "T::Value: Debug")]
pub struct Test6<T: Trait>(Test5<T>);

#[derive(CustomDebug)]
pub struct Test7<T: Trait, U>(#[debug(bound = "T::Value: Debug")] Test5<T>, U);

// Does not implement Debug.
struct NotDebug;

pub trait Trait {
    type Value;
}

impl Trait for NotDebug {
    type Value = u8;
}

fn assert_debug<F: Debug>() {}

fn main() {
    assert!(format!("{:?}", Test0()) == "Test0");
    assert!(format!("{:?}", Test1(0, 0)) == "Test1(0, 0)");
    assert!(format!("{:?}", Test2(0, 0)) == "Test2(0, 0b00000000)");
    assert!(format!("{:?}", Test3(0)) == "Test3(0)");

    assert_debug::<Test4<NotDebug>>();
    assert_debug::<Test5<NotDebug>>();
    assert_debug::<Test6<NotDebug>>();
    assert_debug::<Test7<NotDebug, u8>>();
}
