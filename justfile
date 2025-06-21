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
    cargo set-version   -p nugine-rust-utils    0.3.1

publish:
    # cargo publish     -p nugine-rust-utils    
