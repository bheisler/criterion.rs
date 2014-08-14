RUSTC = rustc -O --cfg ndebug -L target/release/deps -L target/release --out-dir target/release --crate-name test

.PHONY: all bench test

all:
	cargo build --release

bench:
	# TODO Replace with cargo bench
	$(RUSTC) --test src/lib.rs
	target/release/test --bench

test:
	# TODO Replace with cargo bench --test
	$(RUSTC) --test src/lib.rs
	target/release/test
