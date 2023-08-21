dev:
    just check
    just test
    just miri

check:
    cargo fmt
    cargo clippy

test:
    cargo test -p nugine-rust-utils --no-default-features 
    cargo test -p nugine-rust-utils --no-default-features --features alloc
    cargo test -p nugine-rust-utils --no-default-features --features std
    cargo test -p nugine-rust-utils --all-features

    cargo test -p codegen-writer
    cargo test -p bool-logic
    cargo test -p codegen-cfg
    cargo test -p codegen-libc
    
    cargo test -p asc
    # cargo test -p cst-locks
    cargo test -p ordered-vecmap

miri:
    cargo +nightly miri test -p nugine-rust-utils --all-features
    cargo +nightly miri test -p asc
    # cargo +nightly miri test -p cst-locks
    cargo +nightly miri test -p ordered-vecmap

doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --open --all-features

sync-version:
    cargo set-version   -p nugine-rust-utils    0.3.1
    cargo set-version   -p codegen-writer       0.2.0
    cargo set-version   -p bool-logic           0.2.0
    cargo set-version   -p codegen-cfg          0.2.0
    cargo set-version   -p codegen-libc         0.2.1
    cargo set-version   -p asc                  0.1.1
    cargo set-version   -p cst-locks            0.2.0
    cargo set-version   -p ordered-vecmap       0.2.0

publish:
    # cargo publish       -p nugine-rust-utils
    # cargo publish       -p codegen-writer   
    # cargo publish       -p bool-logic
    # cargo publish       -p codegen-cfg      
    # cargo publish       -p codegen-libc     
    # cargo publish       -p asc
    # cargo publish       -p cst-locks
    # cargo publish       -p ordered-vecmap

codegen-libc *ARGS:
    #!/bin/bash -e
    cd {{ justfile_directory() }}
    ./scripts/download-libc.sh
    cargo build -p codegen-libc --features binary --release
    ./target/release/codegen-libc --libc temp/libc {{ ARGS }} | rustfmt
