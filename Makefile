build:
	cargo build --workspace --all-features
check:
	cargo check --all-targets --workspace --all-features
clippy:
	cargo clippy --all-targets --workspace --all-features
doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --workspace --all-features
format:
	cargo +nightly fmt --all
test: format
	cargo clippy --all-targets --workspace --all-features -- -D warnings
	cargo hack test --rust-version --workspace --all-targets \
        --ignore-private --feature-powerset --skip full,default \
        --mutually-exclusive-features std,alloc \
        --mutually-exclusive-features bigint,alloc

b: build
c: clippy
d: doc
f: format
t: test
