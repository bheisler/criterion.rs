SRCS = $(wildcard src/bin/*.rs)
BINS = $(patsubst src/bin/%.rs,target/release/%,$(SRCS))

.PHONY: all doc test

all:
	cargo build --release -u

doc:
	cargo doc

test:
	$(foreach bin,$(BINS),$(bin) &&) true
	./check-line-length.sh
