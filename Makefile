.PHONY: all doc test

all:
	cargo build --release

doc:
	cargo doc

test:
	target/release/fun
	target/release/fun_family
	#target/release/prog
	#target/release/prog_family
	./check-line-length.sh
