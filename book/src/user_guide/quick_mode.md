## Quick mode

Quick mode is enabled with the `--quick` flag and tells criterion to stop benchmarks early once the significance level is below a certail value (default 5%, see the `--significance-level` flag).

Quick mode in criterion works exactly like `tasty-bench` which has a wealth of details: https://github.com/Bodigrim/tasty-bench

### Statistical model

1. Set n ← 1.
1. Measure execution time tₙ of n iterations and execution time t₂ₙ of 2n iterations.
1. Find t which minimizes deviation of (nt, 2nt) from (tₙ, t₂ₙ), namely t ← (tₙ + 2t₂ₙ) / 5n.
1. If deviation is small enough (see `--significance-level`) or time has run out (see `--measurement-time`), return t as a mean execution time.
1. Otherwise set n ← 2n and jump back to Step 2.

### Disclaimer

Statistics is a tricky matter, there is no one-size-fits-all approach. In the absence of a good theory simplistic approaches are as (un)sound as obscure ones. Those who seek statistical soundness should rather collect raw data and process it themselves using a proper statistical toolbox. Data reported by criterion in quick mode is only of indicative and comparative significance.
