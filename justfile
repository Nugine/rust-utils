dev:
    cargo fmt
    cargo clippy
    just test

test:
    cargo test -p nugine-rust-utils
    cargo test -p nugine-rust-utils --features alloc
    cargo test -p nugine-rust-utils --features std

doc:
    cargo doc --no-deps --open

sync-version:
    cargo set-version -p nugine-rust-utils 0.1.0-dev

publish:
    cargo publish -p nugine-rust-utils
