#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse.rs");
    t.pass("tests/02-impl-debug.rs");
    t.pass("tests/03-custom-format.rs");
    t.pass("tests/04-type-parameter.rs");
    t.pass("tests/05-phantom-data.rs");
    t.pass("tests/06-bound-trouble.rs");
    t.pass("tests/07-associated-type.rs");
    t.pass("tests/08-escape-hatch.rs");

    t.pass("tests/09-unit-struct.rs");
    t.pass("tests/10-empty-struct-with-named-fields.rs");
    t.pass("tests/11-tuple-struct.rs");
    t.pass("tests/12-enum.rs");
    t.compile_fail("tests/13-union.rs");
    t.compile_fail("tests/14-invalid-attributes-on-struct.rs");
    t.compile_fail("tests/15-invalid-attributes-on-enum.rs");
    t.pass("tests/16-fields-can-have-different-debug-attributes.rs");
    t.pass("tests/17-bound-debug-attributes-can-appear-after-the-type-already-appeared.rs");
    t.pass("tests/18-bound-debug-attribute-can-contain-multiple-bounds.rs");
    t.pass("tests/18-parse-different-bound-formats.rs");
    t.pass("tests/19-phantom-data-variations.rs");
    t.pass("tests/20-associated-type-variations.rs");
    t.pass("tests/21-reference-bound.rs");
}
