use derive_debug::CustomDebug;
use std::{fmt::Debug, marker::PhantomData};

trait Trait {
    type Value;
}

#[allow(unused_parens, clippy::type_complexity)]
#[derive(CustomDebug)]
struct Test0<'a, T: Trait> {
    field0: T::Value,
    field2: (T::Value),
    field3: &'a T::Value,
    field4: &'a [T::Value],
    field5: Vec<T::Value>,
    field6: (
        T::Value,
        (T::Value),
        &'a T::Value,
        &'a [T::Value],
        Vec<T::Value>,
    ),

    field7: <T as Trait>::Value,
    field8: (<T as Trait>::Value),
    field9: &'a <T as Trait>::Value,
    field10: &'a [<T as Trait>::Value],
    field11: Vec<<T as Trait>::Value>,
    field12: (
        <T as Trait>::Value,
        (<T as Trait>::Value),
        &'a <T as Trait>::Value,
        &'a [<T as Trait>::Value],
        Vec<<T as Trait>::Value>,
    ),
}

#[derive(CustomDebug)]
struct Test1<T>
where
    Vec<T::Value>: Trait,
    T: Trait,
{
    values: Vec<<Vec<<T as Trait>::Value> as Trait>::Value>,
}

#[derive(CustomDebug)]
struct Test2<T>
where
    T: Trait,
    T::Value: Trait,
{
    values: <T::Value as Trait>::Value,
}

#[derive(CustomDebug)]
struct Test3<T>
where
    PhantomData<T>: Trait,
{
    values: <PhantomData<T> as Trait>::Value,
}

// Does not implement Debug.
struct NotDebug;

impl Trait for NotDebug {
    type Value = u8;
}

impl Trait for Vec<u8> {
    type Value = u64;
}

impl Trait for u8 {
    type Value = u32;
}

impl Trait for PhantomData<NotDebug> {
    type Value = u128;
}

fn assert_debug<F: Debug>() {}

fn main() {
    assert_debug::<Test0<NotDebug>>();
    assert_debug::<Test1<NotDebug>>();
    assert_debug::<Test2<NotDebug>>();
    assert_debug::<Test3<NotDebug>>();
}
