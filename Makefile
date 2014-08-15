RUSTC = rustc -O --cfg ndebug -L target/release/deps -L target/release --out-dir target/release --crate-name test

.PHONY: all bench test

all:
	cargo build --release

bench:
	cargo bench

test:
	# Without the filter, this command runs both the tests and the benchmarks!
	cargo bench -- --test ::test::
