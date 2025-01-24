use derive_debug::CustomDebug;
use std::fmt::Debug;

#[derive(CustomDebug)]
#[debug(bound = "for<'b> &'b T: Debug")]
struct Test<'a, T> {
    field: &'a T,
}

struct NoDebug;

impl Debug for &NoDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NoDebug ref")
    }
}

fn main() {
    let no_debug = NoDebug;
    assert!(format!("{:?}", Test { field: &no_debug }) == "Test { field: NoDebug ref }");
}
