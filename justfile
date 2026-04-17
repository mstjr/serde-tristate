test:
    cargo test --all

check:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

publish:
    cargo publish -p serde-tristate-macros
    cargo publish -p serde-tristate
