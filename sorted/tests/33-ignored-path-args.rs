#![allow(dead_code)]
use std::process::ExitCode;

trait TestTrait {
    const VALUE: u32;
}

struct TestStruct<T> {
    value: T,
}

struct A;

impl TestTrait for TestStruct<A> {
    const VALUE: u32 = 0;
}

struct B;

impl TestTrait for TestStruct<B> {
    const VALUE: u32 = 1;
}

#[allow(clippy::match_same_arms)]
#[sorted::check]
fn main() -> ExitCode {
    let x = TestStruct::<B>::VALUE;

    #[sorted]
    match x {
        TestStruct::<B>::VALUE => ExitCode::SUCCESS,
        TestStruct::<A>::VALUE => ExitCode::FAILURE,
        _ => ExitCode::FAILURE,
    }
}
