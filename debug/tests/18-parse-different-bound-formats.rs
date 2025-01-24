use derive_debug::CustomDebug;

trait Trait {
    type Value;
}

#[derive(CustomDebug)]
struct Test0<T: ::core::fmt::Debug> {
    field0: T,
}

#[derive(CustomDebug)]
struct Test1<T>
where
    T: ::core::fmt::Debug,
{
    field0: T,
}

#[derive(CustomDebug)]
struct Test2<'a, T>
where
    for<'b> &'b T: ::core::fmt::Debug,
{
    field0: &'a T,
}

#[derive(CustomDebug)]
struct Test3<'a, 'b: 'a, T> {
    field0: &'a T,
    field1: &'b T,
}

#[derive(CustomDebug)]
struct Test4<'a, 'b, T>
where
    'a: 'b,
{
    field0: &'a T,
    field1: &'b T,
}

#[derive(CustomDebug)]
struct Test5<T>
where
    Self: Trait,
    <Self as Trait>::Value: ::core::fmt::Debug,
{
    field0: <Self as Trait>::Value,
}

impl Trait for Test5<u8> {
    type Value = u8;
}

#[derive(CustomDebug)]
struct Test6<T: Trait>
where
    T::Value: ::core::fmt::Debug,
{
    field0: T::Value,
}

#[derive(CustomDebug)]
struct Test7<T>
where
    T: Trait,
    T::Value: ::core::fmt::Debug,
{
    field0: T::Value,
}

#[derive(CustomDebug)]
struct Test8<T: ::core::fmt::Debug>
where
    Test0<T>: ::core::fmt::Debug,
{
    field0: Test0<T>,
}

#[derive(CustomDebug)]
struct Test9<T>
where
    Test0<T>: ::core::fmt::Debug,
    T: ::core::fmt::Debug,
{
    field0: Test0<T>,
}

fn assert_debug<F: ::core::fmt::Debug>() {}

fn main() {
    assert_debug::<Test0<u8>>();
    assert_debug::<Test1<u8>>();
    assert_debug::<Test2<'static, u8>>();
    assert_debug::<Test3<'static, 'static, u8>>();
    assert_debug::<Test4<'static, 'static, u8>>();
    assert_debug::<Test5<u8>>();
    assert_debug::<Test6<Test5<u8>>>();
    assert_debug::<Test7<Test5<u8>>>();
    assert_debug::<Test8<u8>>();
    assert_debug::<Test9<u8>>();
}
