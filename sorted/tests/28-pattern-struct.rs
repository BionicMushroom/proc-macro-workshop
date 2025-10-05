use std::fmt::Display;

#[allow(dead_code)]
struct TestStruct {
    value: u32,
}

impl Display for TestStruct {
    #[sorted::check]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[sorted]
        match self {
            TestStruct { value: 1 } => write!(f, "1"),
            TestStruct { value: 0 } => write!(f, "0"),
            _ => write!(f, "other"),
        }
    }
}

fn main() {}
