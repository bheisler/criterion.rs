BINDIR = bin
LIB = src/lib.rs
LIBDIR = lib
RUSTC = rustc -O
SRCS = $(wildcard examples/*.rs)
BINS = $(patsubst examples/%.rs,bin/%,$(SRCS))

.PHONY: all bench clean test

all:
	mkdir -p $(LIBDIR)
	$(RUSTC) $(LIB) --out-dir $(LIBDIR)

clean:
	rm -rf bin lib

test:
	rm -rf bin
	mkdir bin
	$(foreach src,$(SRCS),$(RUSTC) $(src) -L $(LIBDIR) --out-dir $(BINDIR);)
	$(foreach bin,$(BINS),$(bin) &&) true
	./check-line-length.sh

bench:
	rm -rf bin
	mkdir bin
	$(RUSTC) --cfg bench --test src/lib.rs --out-dir $(BINDIR)
	RUST_THREADS=1 bin/criterion --test --nocapture
