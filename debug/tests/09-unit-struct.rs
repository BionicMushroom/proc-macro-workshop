use derive_debug::CustomDebug;

#[derive(CustomDebug)]
struct Test;

fn main() {
    assert!(format!("{Test:?}") == "Test");
}
