dev:
    cargo fmt
    cargo clippy
    just test
    just miri

test:
    cd crates/rust-utils && just test

miri:
    cd crates/rust-utils && just miri

doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --open --all-features

sync-version:
    cargo set-version   -p nugine-rust-utils      0.3.1

publish:
    cargo publish       -p nugine-rust-utils
