#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse.rs");
    t.pass("tests/02-create-builder.rs");
    t.pass("tests/03-call-setters.rs");
    t.pass("tests/04-call-build.rs");
    t.pass("tests/05-method-chaining.rs");
    t.pass("tests/06-optional-field.rs");
    t.pass("tests/07-repeated-field.rs");
    t.compile_fail("tests/08-unrecognized-attribute.rs");
    t.pass("tests/09-redefined-prelude-types.rs");

    t.compile_fail("tests/10-generic-struct.rs");
    t.pass("tests/11-unit-struct.rs");
    t.compile_fail("tests/12-tuple-struct.rs");
    t.compile_fail("tests/13-enum.rs");
    t.compile_fail("tests/14-union.rs");
    t.compile_fail("tests/15-name-value-attribute.rs");
    t.compile_fail("tests/16-path-attribute.rs");
    t.compile_fail("tests/17-multiple-correct-attributes.rs");
    t.compile_fail("tests/18-multiple-incorrect-attributes.rs");
    t.compile_fail("tests/19-duplicated-identifier-in-attribute.rs");
    t.compile_fail("tests/20-repeated-non-vec-field.rs");
    t.pass("tests/21-all-vec-paths-in-repeated-fields.rs");
    t.compile_fail("tests/22-repeated-field-with-no-generic-args.rs");
    t.compile_fail("tests/23-repeated-field-with-multiple-generic-args.rs");
    t.pass("tests/24-all-option-paths-in-optional-fields.rs");
    t.pass("tests/25-option-lookalike-fields.rs");
    t.pass("tests/26-generated-methods-can-be-accessed-if-struct-is-pub.rs");
}
