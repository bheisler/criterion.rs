<style type="text/css">
.term-output {
    display: block !important;
    overflow: scroll;
    white-space: pre;
}
body {background-color: black;}
pre {
	font-weight: normal;
	color: #bbb;
	white-space: -moz-pre-wrap;
	white-space: -o-pre-wrap;
	white-space: -pre-wrap;
	white-space: pre-wrap;
	word-wrap: break-word;
	overflow-wrap: break-word;
}
b {font-weight: normal}
b.BOLD {color: #fff}
b.ITA {font-style: italic}
b.UND {text-decoration: underline}
b.STR {text-decoration: line-through}
b.UNDSTR {text-decoration: underline line-through}
b.BLK {color: #000000}
b.RED {color: #aa0000}
b.GRN {color: #00aa00}
b.YEL {color: #aa5500}
b.BLU {color: #0000aa}
b.MAG {color: #aa00aa}
b.CYN {color: #00aaaa}
b.WHI {color: #aaaaaa}
b.HIK {color: #555555}
b.HIR {color: #ff5555}
b.HIG {color: #55ff55}
b.HIY {color: #ffff55}
b.HIB {color: #5555ff}
b.HIM {color: #ff55ff}
b.HIC {color: #55ffff}
b.HIW {color: #ffffff}
b.BBLK {background-color: #000000}
b.BRED {background-color: #aa0000}
b.BGRN {background-color: #00aa00}
b.BYEL {background-color: #aa5500}
b.BBLU {background-color: #0000aa}
b.BMAG {background-color: #aa00aa}
b.BCYN {background-color: #00aaaa}
b.BWHI {background-color: #aaaaaa}
</style>


# Tabulating Results

Criterion can save the results of different benchmark runs and
tabulate the results, making it easier to spot performance changes.

The set of results from a benchmark run is called a `baseline` and each
`baseline` has a name. By default, the most recent run is named `base` but this
can be changed with the `--save-baseline {name}` flag. There's also a special
baseline called `new` which refers to the most recent set of results.

## Comparing profiles

Cargo supports custom
[profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) for
controlling the level of optimizations, debug assertions, overflow checks, and
link-time-optmizations. We can use criterion to benchmark different profiles and
tabulate the results to visualize the changes. Let's use the `base64` crate as
an example:

```bash
> git clone https://github.com/KokaKiwi/rust-hex.git
> cd rust-hex/
```

Now that we've clone the repository, we can generate the first set of benchmark results:

```bash
> cargo bench --profile=release       `# Use the 'release' profile` \
              --bench=hex             `# Select the 'hex' binary` \
              --                      `# Switch args from cargo to criterion` \
              --save-baseline release `# Save the baseline under 'release'`
```

Once the run is complete (this should take a few minutes), we can benchmark the other profile:

```bash
> cargo bench --profile=dev       `# Use the 'dev' profile` \
              --bench=benchmarks  `# Select the 'hex' binary` \
              --                  `# Switch args from cargo to criterion` \
              --save-baseline dev `# Save the baseline under 'dev'`
```

Finally we can compare the two benchmark runs (scroll to the right to see all columns):

```bash
> cargo bench --bench=hex -- --compare --baselines=dev,release
```

<pre class="hljs term-output">group                          dev                                               release
-----                          ---                                               -------
faster_hex_decode              239.50  847.6±16.54µs        ? ?/sec<b class="BOLD"></b><b class=HIG>    1.00      3.5±0.01µs        ? ?/sec</b>
faster_hex_decode_fallback     52.58   567.7±8.36µs        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00     10.8±0.04µs        ? ?/sec</b>
faster_hex_decode_unchecked    400.98   503.7±3.48µs        ? ?/sec<b class="BOLD"></b><b class=HIG>    1.00   1256.2±1.57ns        ? ?/sec</b>
faster_hex_encode              259.95   244.5±2.04µs        ? ?/sec<b class="BOLD"></b><b class=HIG>    1.00    940.5±4.64ns        ? ?/sec</b>
faster_hex_encode_fallback     50.60   565.1±3.41µs        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00     11.2±0.02µs        ? ?/sec</b>
hex_decode                     25.27     3.0±0.01ms        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00    119.3±0.17µs        ? ?/sec</b>
hex_encode                     23.99 1460.8±18.11µs        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00     60.9±0.08µs        ? ?/sec</b>
rustc_hex_decode               28.79     3.1±0.02ms        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00    107.4±0.40µs        ? ?/sec</b>
rustc_hex_encode               25.80  1385.4±4.37µs        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00    53.7±15.63µs        ? ?/sec</b>
</pre>

The first column in the above results has the names of each individual
benchmark. The two other columns (`dev` and `release`) contain the actual
benchmark results. Each baseline column starts with a performance index relative
to the fastest run (eg. `faster_hex_decode` for `dev` has a performance index of
239.50 because it is 239.50 times slower than the `release` build). Next is the
mean execution time plus the standard deviation (eg. 847.6±16.54µs). Lastly
there's an optional throughput. If no throughput data is available, it will be
printed as `? ?/sec`.

## Compact, list view.

If horizontal space is limited or if you're comparing more than two baselines,
it can be convenient to arrange the results in a vertical list rather than in a
table. This can be enabled with the `--compare-list` flag:

```
faster_hex_decode
-----------------
release     1.00       3.5±0.01µs       ? ?/sec
dev       239.50    847.6±16.54µs       ? ?/sec

faster_hex_decode_fallback
--------------------------
release     1.00      10.8±0.04µs       ? ?/sec
dev        52.58     567.7±8.36µs       ? ?/sec

faster_hex_decode_unchecked
---------------------------
release     1.00    1256.2±1.57ns       ? ?/sec
dev       400.98     503.7±3.48µs       ? ?/sec

faster_hex_encode
-----------------
release     1.00     940.5±4.64ns       ? ?/sec
dev       259.95     244.5±2.04µs       ? ?/sec

faster_hex_encode_fallback
--------------------------
release     1.00      11.2±0.02µs       ? ?/sec
dev        50.60     565.1±3.41µs       ? ?/sec

hex_decode
----------
release     1.00     119.3±0.17µs       ? ?/sec
dev        25.27       3.0±0.01ms       ? ?/sec

hex_encode
----------
release     1.00      60.9±0.08µs       ? ?/sec
dev        23.99   1460.8±18.11µs       ? ?/sec

rustc_hex_decode
----------------
release     1.00     107.4±0.40µs       ? ?/sec
dev        28.79       3.1±0.02ms       ? ?/sec

rustc_hex_encode
----------------
release     1.00     53.7±15.63µs       ? ?/sec
dev        25.80    1385.4±4.37µs       ? ?/sec
```

## Filtering results

Some projects have dozens or even hundreds of benchmarks which can be
overwhelming if you're only interested in the performance of a single
feature/function.

Let's clone the `hex` crate and change just a single function:

```bash
> git clone https://github.com/KokaKiwi/rust-hex.git
> cd rust-hex/
```

Save a baseline for the `main` branch:

```bash
> cargo bench --bench=hex          `# Select the 'hex' binary` \
              --                   `# Switch args from cargo to criterion` \
              --save-baseline main `# Save the baseline under 'main'`
```

Create a new branch:

```bash
> git checkout -b new-feature
```

For testing, let's modify the `hex_decode` benchmark to run twice:

```diff
--- a/benches/hex.rs
+++ b/benches/hex.rs
     c.bench_function("hex_decode", |b| {
         let hex = hex::encode(DATA);
-        b.iter(|| hex::decode(&hex).unwrap())
+        b.iter(|| (hex::decode(&hex).unwrap(),hex::decode(&hex).unwrap()))
     });
```

Now we can benchmark just the `hex_decode` function:

```bash
> cargo bench --bench=hex                 `# Select the 'hex' binary` \
              --                          `# Switch args from cargo to criterion` \
              --save-baseline new-feature `# Save the baseline under 'new-feature'` \
              ^hex_decode                 `# Select the 'hex_decode' benchmark`
```

And compare it to the `main` branch, verifying that we've introduced a 2x
performance regression:

```bash
> cargo bench --bench=hex -- --compare --baselines=main,new-feature ^hex_decode
```

<pre class="hljs term-output">group                   main                                      new-feature
-----                   ----                                      -----------
hex_decode<b class="BOLD"></b><b class=HIG>    1.00    119.1±1.30µs        ? ?/sec</b>    2.06    245.5±2.21µs        ? ?/sec
</pre>

## Thresholds

If we don't know which benchmarks are of interest, we can filter the results
based on how much they've changed.

In the previous section, we only generated results for the `hex_decode`
benchmark. For this run, we need a complete set of results:

```bash
> cargo bench --bench=hex                 `# Select the 'hex' binary` \
              --                          `# Switch args from cargo to criterion` \
              --save-baseline new-feature `# Save the baseline under 'new-feature'` \
```

Now we can compare the results that differ by more than 10%:

```bash
> cargo bench --bench=hex -- --compare --baselines=main,new-feature --compare-threshold=10
```

<pre class="hljs term-output">group                   main                                      new-feature
-----                   ----                                      -----------
hex_decode<b class="BOLD"></b><b class=HIG>    1.00    119.1±1.30µs        ? ?/sec</b>    2.02    240.0±1.05µs        ? ?/sec
</pre>

The above console output shows that only a single benchmark changed by more than
10%.

## Importing/Exporting JSON

Baselines can be saved in JSON files for later use with the `--export` flag. Continuing with the `hex` crate example, here's how to
save the `release` and `dev` baselines as JSON:

```bash
> cargo bench --bench=hex -- --export release > release.json
```

```bash
> cargo bench --bench=hex -- --export dev > dev.json
```

Baselines stored as JSON can be referenced directly when comparing results:

```bash
> cargo bench --bench=hex -- --compare --baselines dev.json,release.json
```

<pre class="hljs term-output">group                          dev                                               release
-----                          ---                                               -------
faster_hex_decode              239.50  847.6±16.54µs        ? ?/sec<b class="BOLD"></b><b class=HIG>    1.00      3.5±0.01µs        ? ?/sec</b>
faster_hex_decode_fallback     52.58   567.7±8.36µs        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00     10.8±0.04µs        ? ?/sec</b>
faster_hex_decode_unchecked    400.98   503.7±3.48µs        ? ?/sec<b class="BOLD"></b><b class=HIG>    1.00   1256.2±1.57ns        ? ?/sec</b>
faster_hex_encode              259.95   244.5±2.04µs        ? ?/sec<b class="BOLD"></b><b class=HIG>    1.00    940.5±4.64ns        ? ?/sec</b>
faster_hex_encode_fallback     50.60   565.1±3.41µs        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00     11.2±0.02µs        ? ?/sec</b>
hex_decode                     25.27     3.0±0.01ms        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00    119.3±0.17µs        ? ?/sec</b>
hex_encode                     23.99 1460.8±18.11µs        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00     60.9±0.08µs        ? ?/sec</b>
rustc_hex_decode               28.79     3.1±0.02ms        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00    107.4±0.40µs        ? ?/sec</b>
rustc_hex_encode               25.80  1385.4±4.37µs        ? ?/sec<b class="BOLD"></b><b class=HIG>     1.00    53.7±15.63µs        ? ?/sec</b>
</pre>
