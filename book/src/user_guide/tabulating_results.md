# Tabulating Results

Criterion can save the results of different benchmark runs and
tabulate the results, making it easier to spot performance changes.

The set of results from a benchmark run is called a `baseline` and each `baseline` has a name. By default, the most recent run is named `"base"` but this can be changed with the `--save-baseline {name}` flag. There's also a special baseline called `"new"` which refers to the most recent set of results.

## Comparing branches

## Comparing profiles

Cargo supports custom [profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) for controlling the level of optimizations, debug assertions, overflow checks, and link-time-optmizations. We can use criterion to benchmark different profiles and tabulate the results to visualize the changes. Let's use the `base64` crate as an example:

```bash
> git clone https://github.com/marshallpierce/rust-base64.git
> cd rust-base64/
```

Now that we've clone the repository, we can generate the first set of benchmark results:

```bash
> cargo bench --profile=release       `# Use the 'release' profile` \
              --bench=benchmarks      `# Select the 'benchmarks' binary` \
              --                      `# Switch args from cargo to criterion` \
              --save-baseline release `# Save the baseline under 'release'`
```

Once the run is complete (this should take 10-20 minutes), we can benchmark the other profile:

```bash
> cargo bench --profile=dev       `# Use the 'dev' profile` \
              --bench=benchmarks  `# Select the 'benchmarks' binary` \
              --                  `# Switch args from cargo to criterion` \
              --save-baseline dev `# Save the baseline under 'dev'`
```


Color test:

<style type="text/css">
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

<pre class="hljs" style="display:block">appveyor.yml    Cargo.toml       LICENSE-MIT  <b class=HIB>target</b>
<b class=HIB>bencher_compat</b>  CHANGELOG.md     <b class=HIB>macro</b>        <b class=HIB>tests</b>
<b class=HIB>benches</b>         <b class=HIB>ci</b>               <b class=HIB>plot</b>
<b class=HIB>book</b>            CONTRIBUTING.md  README.md
<b class=HIG>Cargo.lock</b>      LICENSE-APACHE   <b class=HIB>src</b></pre>
