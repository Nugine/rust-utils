set fallback

[no-cd]
test:
    cargo test

[no-cd]
miri:
    cargo +nightly miri test
