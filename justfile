dev:
    cargo fmt
    cargo clippy
    just test
    just miri

test:
    cargo test -p nugine-rust-utils --no-default-features 
    cargo test -p nugine-rust-utils --no-default-features --features alloc
    cargo test -p nugine-rust-utils --no-default-features --features std
    cargo test -p nugine-rust-utils --all-features

miri:
    cargo +nightly miri test -p nugine-rust-utils --all-features

doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --open --all-features

sync-version:
    cargo set-version   -p nugine-rust-utils      0.3.1

publish:
    cargo publish       -p nugine-rust-utils
