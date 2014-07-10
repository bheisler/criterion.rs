SRCS = $(wildcard examples/*.rs)
BINS = $(patsubst examples/%.rs,target/release/%,$(SRCS))

.PHONY: all test

all:
	cargo build --release

test:
	#$(foreach bin,$(BINS),$(bin) &&) true
	target/release/fib
	target/release/pyclock
	target/release/pyfib
	./check-line-length.sh
