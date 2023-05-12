dev:
    cargo fmt
    cargo clippy
    cargo test

doc:
    cargo doc --no-deps --open

sync-version:
    cargo set-version -p nugine-rust-utils 0.1.0-dev

publish:
    cargo publish -p nugine-rust-utils
