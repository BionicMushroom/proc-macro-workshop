use seq::seq;

fn main() {
    let test = seq!(_ in 0..4 {
       (#("abc",)*)
    });

    let mut appended = String::new();
    appended.extend([test.0, test.1, test.2, test.3]);

    assert!(appended == "abcabcabcabc");
}
