use ui_test::{
    default_file_filter, default_per_file_config, dependencies::DependencyBuilder,
    run_tests_generic, status_emitter, Args, Config,
};

fn main() -> ui_test::Result<()> {
    let mut config = Config {
        output_conflict_handling: if std::env::var_os("BLESS").is_some() {
            ui_test::bless_output_files
        } else {
            ui_test::error_on_output_conflict
        },
        ..Config::rustc("tests/ui")
    };
    config.comment_defaults.base().set_custom(
        "dependencies",
        DependencyBuilder {
            crate_manifest_path: "tests/deps/Cargo.toml".into(),
            ..DependencyBuilder::default()
        },
    );

    // Don't compare exact stderr output against .stderr files.
    // Only check inline `//~` error annotations, which are
    // resilient to diagnostic wording changes across Rust versions.
    config.stderr_filter("(?s).*", "");

    let args = Args::test()?;
    config.with_args(&args);

    run_tests_generic(
        vec![config],
        default_file_filter,
        default_per_file_config,
        status_emitter::Text::verbose(),
    )
}
