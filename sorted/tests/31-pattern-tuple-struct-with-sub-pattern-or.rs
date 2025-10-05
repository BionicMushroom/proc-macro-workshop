use std::fmt::Display;

#[allow(dead_code)]
struct TestStruct(u32);

impl Display for TestStruct {
    #[allow(clippy::unnested_or_patterns, unused_parens)]
    #[sorted::check]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[sorted]
        match self {
            TestStruct(0) => write!(f, "0"),
            TestStruct((5 | 6) | (1 | 2)) => write!(f, "1, 2, 5 or 6"),
            _ => write!(f, "other"),
        }
    }
}

fn main() {}
