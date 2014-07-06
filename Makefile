SRCS = $(wildcard examples/*.rs)
BINS = $(patsubst examples/%.rs,target/%,$(SRCS))

.PHONY: all bench test

all:
	cargo build

test:
	#$(foreach bin,$(BINS),$(bin) &&) true
	target/fib
	./check-line-length.sh

#bench:
	#rm -rf bin
	#mkdir bin
	#$(RUSTC) --cfg bench --test src/lib.rs --out-dir $(BINDIR)
	#RUST_THREADS=1 bin/criterion --test --nocapture
