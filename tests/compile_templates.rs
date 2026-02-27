#[test]
fn validates_template_literals_at_compile_time() {
    let cases = trybuild::TestCases::new();

    cases.pass("tests/ui/pass_templates.rs");
    cases.compile_fail("tests/ui/fail_unmatched_open.rs");
    cases.compile_fail("tests/ui/fail_unmatched_close.rs");
    cases.compile_fail("tests/ui/fail_empty_placeholder.rs");
    cases.compile_fail("tests/ui/fail_indexed_placeholder.rs");
    cases.compile_fail("tests/ui/fail_format_specifier.rs");
}
