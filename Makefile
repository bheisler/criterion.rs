SRCS = $(wildcard examples/*.rs)
BINS = $(patsubst examples/%.rs,target/%,$(SRCS))

.PHONY: all test

all:
	cargo build

test:
	#$(foreach bin,$(BINS),$(bin) &&) true
	target/fib
	./check-line-length.sh
