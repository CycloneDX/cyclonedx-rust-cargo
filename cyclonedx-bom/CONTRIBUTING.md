# Contributing

Pull requests are welcome.
But please read the
[CycloneDX contributing guidelines](https://github.com/CycloneDX/.github/blob/master/CONTRIBUTING.md)
first.

## Build

```shell
cargo +stable build --verbose
```

## Test

Run the tests:

```shell
cargo test
```

## Coding standards

Check for deviations from coding standards:

```shell
cargo fmt -- --check
cargo clippy --all-targets
```

Apply coding standards via:

```shell
cargo fmt
```

## Sign off your commits

Please sign off your commits,
to show that you agree to publish your changes under the current terms and licenses of the project.

```shell
git commit --signoff ...
```
