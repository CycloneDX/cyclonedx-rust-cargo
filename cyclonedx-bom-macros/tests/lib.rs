#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
    // TODO: migrate away from `trybuild` crate and re-enable
    // The `trybuild` crate is too limited for our purposes.
    // We cannot use it to run tests on both MSRV and latest Rust version.
    // Unlike e.g. `ui-test` it does not support different expected outputs per Rust version,
    // and neither does it support checking that the build fails without matching the exact error message.
    // And while it tries to compensate for some of the differences we cannot even use
    // a recent version which does that because it won't compile on our MSRV!
    // t.compile_fail("tests/ui/fail/*.rs");
}
