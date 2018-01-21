# Contributing to Criterion.<span></span>rs

## Ideas, Experiences and Questions

The easiest way to contribute to Criterion.<span></span>rs is to use it and report your experiences, ask questions and contribute ideas. We'd love to hear your thoughts on how to make Criterion.<span></span>rs better, or your comments on why you are or are not currently using it.

Issues, ideas, requests and questions should be posted on the issue tracker at:

https://github.com/japaric/criterion.rs/issues

## Code

Pull requests are welcome, though please raise an issue for discussion first if none exists. We're happy to assist new contributors.

If you're not sure what to work on, try checking the [good first issue label](https://github.com/japaric/criterion.rs/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

To make changes to the code, fork the repo and clone it:

`git clone git@github.com:your-username/criterion.rs.git`

You'll probably want to install [gnuplot](http://www.gnuplot.info/) as well. See the gnuplot website for installation instructions.

Then make your changes to the code. When you're done, run the tests:

```
cargo test --all
cargo bench
```

It's a good idea to run clippy and fix any warnings as well:

```
cargo +nightly install clippy
cargo +nightly clippy --all
```

Finally, run Rustfmt to maintain a common code style:

```
rustup component add rustfmt-preview --toolchain=nightly
cargo +nightly fmt
```

Don't forget to update the CHANGELOG.md file and any appropriate documentation. Once you're finished, push to your fork and submit a pull request. We try to respond to new issues and pull requests quickly, so if there hasn't been any response for more than a few days feel free to ping @bheisler.

Some things that will increase the chance that your pull request is accepted:

* Write tests
* Clearly document public methods
* Write a good commit message

## Code of Conduct

We follow the [Rust Code of Conduct](http://www.rust-lang.org/conduct.html).
