use analysis;
use benchmark::{BenchmarkConfig, PartialBenchmarkConfig};
use measurement::Measurement;
use report::BenchmarkId as InternalBenchmarkId;
use report::ReportContext;
use routine::Function;
use {Bencher, Criterion};

// TODO: Add all the configuration stuff to BenchmarkGroup.
// TODO: build out bench_custom and bench_custom_with_input

/// TODO
pub struct BenchmarkGroup<'a, M: Measurement> {
    criterion: &'a mut Criterion<M>,
    group_name: String,
    all_ids: Vec<InternalBenchmarkId>,
    any_matched: bool,
    partial_config: PartialBenchmarkConfig,
    final_config: Option<BenchmarkConfig>,
    report_context: Option<ReportContext>,
}
impl<'a, M: Measurement> BenchmarkGroup<'a, M> {
    pub(crate) fn new(criterion: &mut Criterion<M>, group_name: String) -> BenchmarkGroup<M> {
        BenchmarkGroup {
            criterion,
            group_name,
            all_ids: vec![],
            any_matched: false,
            partial_config: PartialBenchmarkConfig::default(),
            final_config: None,
            report_context: None,
        }
    }

    /// TODO
    pub fn bench_function<ID: IntoBenchmarkId, F>(&mut self, id: ID, mut f: F) -> &mut Self
    where
        F: FnMut(&mut Bencher<M>),
    {
        self.finalize_config();
        self.run_bench(id.into_benchmark_id(), &(), |b, _| f(b));
        self
    }

    /// TODO
    pub fn bench_with_input<ID: IntoBenchmarkId, F, I>(
        &mut self,
        id: ID,
        input: &I,
        f: F,
    ) -> &mut Self
    where
        F: FnMut(&mut Bencher<M>, &I),
    {
        self.finalize_config();
        self.run_bench(id.into_benchmark_id(), input, f);
        self
    }

    fn run_bench<F, I>(&mut self, id: BenchmarkId, input: &I, f: F)
    where
        F: FnMut(&mut Bencher<M>, &I),
    {
        // TODO: Fix throughput
        let throughput = None; //self.throughput.as_ref().map(|func| func(value));
        let mut id = InternalBenchmarkId::new(
            self.group_name.clone(),
            id.function_name,
            id.parameter,
            throughput.clone(),
        );

        assert!(
            !self.all_ids.contains(&id),
            "Benchmark IDs must be unique within a group."
        );

        id.ensure_directory_name_unique(&self.criterion.all_directories);
        self.criterion
            .all_directories
            .insert(id.as_directory_name().to_owned());
        id.ensure_title_unique(&self.criterion.all_titles);
        self.criterion.all_titles.insert(id.as_title().to_owned());

        if self.criterion.filter_matches(id.id()) {
            self.any_matched = true;

            let mut func = Function::new(f);

            analysis::common(
                &id,
                &mut func,
                self.final_config.as_ref().unwrap(),
                self.criterion,
                self.report_context.as_ref().unwrap(),
                input,
                throughput,
            );
        }

        self.all_ids.push(id);
    }

    fn finalize_config(&mut self) {
        if self.final_config.is_some() {
            return;
        }

        let config = self.partial_config.to_complete(&self.criterion.config);
        self.final_config = Some(config);
        self.report_context = Some(ReportContext {
            output_directory: self.criterion.output_directory.clone(),
            plotting: self.criterion.plotting,
            plot_config: self.partial_config.plot_config.clone(),
            test_mode: self.criterion.test_mode,
        })
    }

    /// TODO
    pub fn finish(self) {
        ::std::mem::drop(self);
    }
}
impl<'a, M: Measurement> Drop for BenchmarkGroup<'a, M> {
    fn drop(&mut self) {
        // I don't really like having a bunch of non-trivial code in drop, but this is the only way
        // to really write linear types like this in Rust...
        if self.all_ids.len() > 1
            && self.any_matched
            && self.criterion.profile_time.is_none()
            && !self.criterion.test_mode
        {
            self.criterion.report.summarize(
                self.report_context.as_ref().unwrap(),
                &self.all_ids,
                self.criterion.measurement.formatter(),
            );
        }
        if self.any_matched {
            println!();
        }
    }
}

/// TODO
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct BenchmarkId {
    pub(crate) function_name: Option<String>,
    pub(crate) parameter: Option<String>,
}
impl BenchmarkId {
    /// Construct a new benchmark ID from a string function name and a parameter value.
    ///
    /// Note that the parameter value need not be the same as the parameter passed to your
    /// actual benchmark. For instance, you might have a benchmark that takes a 1MB string as
    /// input. It would be impractical to embed the whole string in the benchmark ID, so instead
    /// your parameter value might be a descriptive string like "1MB Alphanumeric".
    ///
    /// # Examples
    /// ```
    /// # use criterion::{BenchmarkId, Criterion};
    /// // A basic benchmark ID is typically constructed from a constant string and a simple
    /// // parameter
    /// let basic_id = BenchmarkId::new("my_id", 5);
    ///
    /// // The function name can be a string
    /// let function_name = "test_string".to_string();
    /// let string_id = BenchmarkId::new(function_name, 12);
    ///
    /// // Benchmark IDs are passed to benchmark groups:
    /// let mut criterion = Criterion::default();
    /// let mut group = criterion.benchmark_group("My Group");
    /// // Generate a very large input
    /// let input : String = ::std::iter::repeat("X").take(1024 * 1024).collect();
    ///
    /// // Note that we don't have to use the input as the parameter in the ID
    /// group.bench_with_input(BenchmarkId::new("Test long string", "1MB X's"), &input, |b, i| {
    ///     b.iter(|| i.len())
    /// });
    /// ```
    pub fn new<S: Into<String>, P: ::std::fmt::Display>(
        function_name: S,
        parameter: P,
    ) -> BenchmarkId {
        BenchmarkId {
            function_name: Some(function_name.into()),
            parameter: Some(format!("{}", parameter)),
        }
    }

    pub(crate) fn no_function() -> BenchmarkId {
        BenchmarkId {
            function_name: None,
            parameter: None,
        }
    }

    pub(crate) fn no_function_with_input<P: ::std::fmt::Display>(parameter: P) -> BenchmarkId {
        BenchmarkId {
            function_name: None,
            parameter: Some(format!("{}", parameter)),
        }
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::BenchmarkId {}
    impl<S: Into<String>> Sealed for S {}
}

/// Sealed trait which allows users to automatically convert strings to benchmark IDs.
pub trait IntoBenchmarkId: private::Sealed {
    fn into_benchmark_id(self) -> BenchmarkId;
}
impl IntoBenchmarkId for BenchmarkId {
    fn into_benchmark_id(self) -> BenchmarkId {
        self
    }
}
impl<S: Into<String>> IntoBenchmarkId for S {
    fn into_benchmark_id(self) -> BenchmarkId {
        BenchmarkId {
            function_name: Some(self.into()),
            parameter: None,
        }
    }
}
