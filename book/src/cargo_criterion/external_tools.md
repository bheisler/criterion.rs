# External Tools

cargo-criterion provides a machine-readable output stream which other tools can consume to collect
information about the Criterion.rs benchmarks.

To enable this output stream, pass the `--message-format` argument when running cargo-criterion.

## JSON messages

When passing `--message-format=json` cargo-criterion will output information about:

* Benchmarks, including the basic statistics about the measurements
* Benchmark groups

The output goes to stdout, with one JSON object per line. The `reason` field distinguishes different
kinds of messages.

Additional messages or fields may be added to the output in the future.

### Benchmark Complete Messages

The "benchmark-complete" message includes the measurements and basic statistics from a single 
Criterion.rs benchmark. The message format is as follows:

```json
{
  /* The "reason" indicates which kind of message this is. */
  "reason": "benchmark-complete",
  /* The id is the identifier of this benchmark */
  "id": "norm",
  /* Path to the directory containing the report for this benchmark */
  "report_directory": "target/criterion/reports/norm",
  /* List of integer iteration counts */
  "iteration_count": [
    30,
    /* ... */
    3000
  ],
  /* List of floating point measurements (eg. time, CPU cycles) taken 
  from the benchmark */
  "measured_values": [
    124200.0,
    /* ... */
    9937100.0
  ],
  /* The unit associated with measured_values. */
  "unit": "ns",
  /* The throughput value associated with this benchmark. This can be used 
  to calculate throughput rates, eg. in bytes or elements per second. */
  "throughput": [
    {
      "per_iteration": 1024,
      "unit": "elements"
    }
  ],
  /* Confidence intervals for the basic statistics that cargo-criterion 
  computes. */
  /* 
  "typical" is either the slope (if available) or the mean (if not). It
  makes a good general-purpose estimate of the typical performance of a
  function.
  */
  "typical": {
    "estimate": 3419.4923993891925,
    "lower_bound": 3375.24221103098,
    "upper_bound": 3465.458469579234,
    "unit": "ns"
  },
  "mean": {
    "estimate": 3419.5340743105917,
    "lower_bound": 3374.4765622217083,
    "upper_bound": 3474.096214164006,
    "unit": "ns"
  },
  "median": {
    "estimate": 3362.8249818445897,
    "lower_bound": 3334.259259259259,
    "upper_bound": 3387.5146198830407,
    "unit": "ns"
  },
  "median_abs_dev": {
    "estimate": 130.7846461816652,
    "lower_bound": 96.55619525548211,
    "upper_bound": 161.1643711235156,
    "unit": "ns"
  },
  
  /* Note that not all benchmarks can measure the slope, so it may be 
  missing. */
  "slope": {
    "estimate": 3419.4923993891925,
    "lower_bound": 3375.24221103098,
    "upper_bound": 3465.458469579234,
    "unit": "ns"
  },

  /* "change" contains some additional statistics about the difference 
  between this run and the last */
  "change": {
    /* Percentage differences in the mean & median values */
    "mean": {
      "estimate": 0.014278477848724602,
      "lower_bound": -0.01790259435189548,
      "upper_bound": 0.03912764721581533,
      "unit": "%"
    },
    "median": {
      "estimate": 0.012211662837601445,
      "lower_bound": -0.0005448009516478807,
      "upper_bound": 0.024243170768727857,
      "unit": "%"
    },
    /* 
    Indicates whether cargo-criterion found a statistically-significant 
    change. Values are NoChange, Improved, or Regressed
    */
    "change": "NoChange"
  }
}
```

### Group Complete Messages

When a benchmark group is completed, cargo-criterion emits a "group-complete" message containing
some information about the group.

```json
{
  "reason": "group-complete",
  /* The name of the benchmark group */
  "group_name": "throughput",
  /* List of the benchmark IDs in this group */
  "benchmarks": [
    "throughput/Bytes",
    "throughput/Bytes",
    "throughput/Elem"
  ],
  /* Path to the directory that contains the report for this group */
  "report_directory": "target/criterion/reports/throughput"
}
```