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

    cargo test -p codegen-writer
    cargo test -p bool-logic
    cargo test -p codegen-cfg
    cargo test -p codegen-libc

miri:
    cargo +nightly miri test -p nugine-rust-utils --all-features

doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --open --all-features

sync-version:
    cargo set-version   -p nugine-rust-utils    0.3.1
    cargo set-version   -p codegen-writer       0.2.0
    cargo set-version   -p bool-logic           0.2.0
    cargo set-version   -p codegen-cfg          0.2.0
    cargo set-version   -p codegen-libc         0.2.1

publish:
    # cargo publish       -p nugine-rust-utils
    # cargo publish       -p codegen-writer   
    # cargo publish       -p bool-logic
    # cargo publish       -p codegen-cfg      
    # cargo publish       -p codegen-libc     

codegen-libc *ARGS:
    #!/bin/bash -e
    cd {{ justfile_directory() }}
    ./scripts/download-libc.sh
    cargo build -p codegen-libc --features binary --release
    ./target/release/codegen-libc --libc temp/libc {{ ARGS }} | rustfmt
