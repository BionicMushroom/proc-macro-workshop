use derive_debug::CustomDebug;
use std::{
    fmt::Debug,
    marker::{self, PhantomData},
};

#[allow(unused_parens, clippy::type_complexity)]
#[derive(CustomDebug)]
struct Test0<'a, T> {
    field0: PhantomData<T>,
    field1: marker::PhantomData<T>,
    field2: std::marker::PhantomData<T>,
    field3: ::std::marker::PhantomData<T>,
    field4: PhantomData<[T]>,
    field5: PhantomData<(T)>,
    field6: PhantomData<&'a T>,
    field7: PhantomData<&'a [T]>,
    field8: PhantomData<Vec<T>>,
    field9: PhantomData<(T, (T), &'a T, &'a [T], Vec<T>, [T])>,
}

// Does not implement Debug.
struct NotDebug;

mod test0 {
    use super::CustomDebug;

    #[derive(CustomDebug)]
    struct PhantomData;

    #[derive(CustomDebug)]
    pub struct Test0 {
        field0: PhantomData,
    }

    mod marker {
        use super::CustomDebug;

        #[derive(CustomDebug)]
        pub struct PhantomData<T, U> {
            field0: T,
            field1: U,
        }
    }

    #[derive(CustomDebug)]
    pub struct Test1<T, U> {
        field0: marker::PhantomData<T, U>,
    }
}

fn assert_debug<F: Debug>() {}

fn main() {
    assert_debug::<Test0<NotDebug>>();
    assert_debug::<test0::Test0>();
    assert_debug::<test0::Test1<u32, u64>>();
}
