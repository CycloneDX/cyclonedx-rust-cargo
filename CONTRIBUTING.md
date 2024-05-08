# Contributing

Contributions are welcome!

But please read the
[CycloneDX contributing guidelines](https://github.com/CycloneDX/.github/blob/master/CONTRIBUTING.md)
first.

## Reporting an issue

This project uses GitHub issues to manage the issues. Open an issue directly in GitHub.

If you believe you found a bug, and it's likely possible, please indicate a way to reproduce it, what you are seeing and what you would expect to see.

## Asking questions

We have a `#rust-cargo` Channel in the CycloneDX Slack (link in the [`README.md`](README.md)).

## Pull Requests

Pull requests are welcome.
Please follow the steps outlined below and make sure to check clippy, format the code and check test output.

### Sign off your commits

Please sign off your commits,
to show that you agree to publish your changes under the current terms and licenses of the project.

```shell
git commit --signoff ...
```
   
## Building and developing the project

### Build

```shell
cargo +stable build --verbose
```

### Test

Run the tests:

```shell
cargo test
```

### Coding standards

Check for deviations from coding standards:

```shell
cargo fmt -- --check
cargo clippy --all-targets --workspace --tests
```

Apply coding standards via:

```shell
cargo fmt
```
