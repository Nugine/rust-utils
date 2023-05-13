dev:
    cargo fmt
    cargo clippy
    just test

test:
    cargo test -p nugine-rust-utils --no-default-features 
    cargo test -p nugine-rust-utils --no-default-features --features alloc
    cargo test -p nugine-rust-utils --no-default-features --features std
    cargo test -p nugine-rust-utils --all-features

doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --open --all-features

sync-version:
    cargo set-version   -p nugine-rust-utils      0.1.4-dev

publish:
    cargo publish       -p nugine-rust-utils
