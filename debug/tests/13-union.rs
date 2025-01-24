use derive_debug::CustomDebug;

#[derive(CustomDebug)]
union Test {
    field0: u32,
    field1: f32,
}

fn main() {}
