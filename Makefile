build:
	cargo build --workspace --all-features
check:
	cargo check --workspace --all-features
clippy:
	cargo clippy --workspace --all-features
doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --workspace --all-features --no-deps
test:
	cargo +nightly fmt --all
	cargo clippy --workspace --all-features -- -D warnings
	cargo hack check --rust-version --workspace --all-targets --ignore-private --feature-powerset
