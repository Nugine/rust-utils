# ----- shared recipes -----

[no-cd]
dev:
    just check
    just test
    just miri

[no-cd]
doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --open --all-features

[no-cd]
check:
    cargo fmt
    cargo clippy

# ----- workspace recipes -----

test:
    #!/bin/bash -ex
    for c in `ls crates`; do
        pushd crates/$c
            just test
        popd
    done

miri:
    #!/bin/bash -ex
    for c in `ls crates`; do
        pushd crates/$c
            just miri
        popd
    done


sync-version:
    cargo set-version   -p bool-logic           0.2.0
    cargo set-version   -p codegen-cfg          0.2.0
    cargo set-version   -p codegen-libc         0.2.1
    cargo set-version   -p nugine-rust-utils    0.3.1

publish:
    # cargo publish     -p bool-logic           
    # cargo publish     -p codegen-cfg          
    # cargo publish     -p codegen-libc         
    # cargo publish     -p nugine-rust-utils    

# ----- special recipes -----

codegen-libc *ARGS:
    #!/bin/bash -e
    cd {{ justfile_directory() }}
    ./scripts/download-libc.sh
    cargo build -p codegen-libc --features binary --release
    ./target/release/codegen-libc --libc temp/libc {{ ARGS }} | rustfmt
