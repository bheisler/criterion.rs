# Criterion.rs #

Criterion.rs is a statistics-driven micro-benchmarking tool. It is a Rust port of [Haskell's Criterion](https://hackage.haskell.org/package/criterion) library.

Criterion.rs benchmarks collect and store statistical information from run to run and can automatically detect performance regressions as well as measuring optimizations.

Criterion.rs is free and open source. You can find the source on [GitHub](https://github.com/japaric/criterion.rs). Issues and feature requests can be posted on [the issue tracker](https://github.com/japaric/criterion.rs/issues).

## API Docs ##

In addition to this book, you may also wish to read [the API documentation](http://japaric.github.io/criterion.rs/criterion/).

## License ##

Criterion.rs is dual-licensed under the [Apache 2.0](https://github.com/japaric/criterion.rs/blob/master/LICENSE-APACHE) and the [MIT](https://github.com/japaric/criterion.rs/blob/master/LICENSE-MIT) licenses.

## Debug Output ##

To enable debug output in Criterion.rs, define the environment variable `CRITERION_DEBUG`. For example (in bash):

```bash
CRITERION_DEBUG=1 cargo bench
```

This will enable extra debug output. Criterion.rs will also save the gnuplot scripts alongside the generated plot files. When raising issues with Criterion.rs (especially when reporting issues with the plot generation) please run your benchmarks with this option enabled and provide the additional output and relevant gnuplot scripts.