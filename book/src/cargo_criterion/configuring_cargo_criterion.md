# Configuring cargo-criterion

cargo-criterion can be configured by placing a `Criterion.toml` file in your crate, alongside your
`Cargo.toml`.

The available settings are documented below:

```toml
# This is used to override the directory where cargo-criterion saves 
# its data and generates reports.
criterion_home = "./target/criterion"

# This is used to configure the format of cargo-criterion's command-line output.
# Options are:
# criterion: Prints confidence intervals for measurement and throughput, and 
#   indicates whether a change was detected from the previous run. The default.
# quiet: Like criterion, but does not indicate changes. Useful for simply 
#   presenting output numbers, eg. on a library's README.
# verbose: Like criterion, but prints additional statistics.
# bencher: Emulates the output format of the bencher crate and nightly-only 
#   libtest benchmarks.
output_format = "criterion"

# This is used to configure the plotting backend used by cargo-criterion. 
# Options are "gnuplot" and "plotters", or "auto", which will use gnuplot if it's
# available or plotters if it isn't.
ploting_backend = "auto"

# The colors table allows users to configure the colors used by the charts 
# cargo-criterion generates.
[colors]
# These are used in many charts to compare the current measurement against 
# the previous one.
current_sample = {r = 31, g = 120, b = 180}
previous_sample = {r = 7, g = 26, b = 28}

# These are used by the full PDF chart to highlight which samples were outliers.
not_an_outlier = {r = 31, g = 120, b = 180}
mild_outlier = {r = 5, g = 127, b = 0}
severe_outlier = {r = 7, g = 26, b = 28}

# These are used for the line chart to compare multiple different functions.
comparison_colors = [
    {r = 8, g = 34, b = 34},
    {r = 6, g = 139, b = 87},
    {r = 0, g = 139, b = 139},
    {r = 5, g = 215, b = 0},
    {r = 0, g = 0, b = 139},
    {r = 0, g = 20, b = 60},
    {r = 9, g = 0, b = 139},
    {r = 0, g = 255, b = 127},
]

```