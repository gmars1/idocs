# Development

## Build

```
cargo build --release
```

Binary is placed at `target/release/idocs`.

## Test

```
cargo test
```

Integration tests are in `tests/integration.rs`. They test every command,
staleness detection, JSON output, exit codes, and error paths.


