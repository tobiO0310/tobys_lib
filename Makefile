build:
	cargo build --workspace --all-features
check:
	cargo check --workspace --all-features
clippy:
	cargo clippy --workspace --all-features
doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --workspace --all-features --no-deps
format:
	cargo +nightly fmt --all
test: format
	cargo clippy --workspace --all-features -- -D warnings
	cargo hack check --rust-version --workspace --all-targets --ignore-private --feature-powerset --skip full

b: build
c: check clippy
d: doc
f: format
t: test