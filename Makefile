SRCS = $(wildcard src/bin/*.rs)
BINS = $(patsubst src/bin/%.rs,target/release/%,$(SRCS))

.PHONY: all test

all:
	cargo build --release

test:
	$(foreach bin,$(BINS),$(bin) &&) true
	./check-line-length.sh
