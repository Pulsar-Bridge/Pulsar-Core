# Contributing to pulsar-core

## PR checklist

Before opening a PR, all of the following must be clean:

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets
cargo test
```

Additionally, for any PR that touches a DB connection, an auth path, or a new
endpoint: