use crate::benchmark::BenchmarkConfig;
use crate::connection::OutgoingMessage;
use crate::measurement::Measurement;
use crate::report::{BenchmarkId, Report, ReportContext};
use crate::{ActualSamplingMode, Bencher, Criterion, DurationExt};
use std::marker::PhantomData;
use std::time::Duration;

/// PRIVATE
pub(crate) trait Routine<M: Measurement, T: ?Sized> {
    /// PRIVATE
    fn bench(&mut self, m: &M, iters: &[u64], parameter: &T) -> Vec<f64>;
    /// PRIVATE
    fn warm_up(&mut self, m: &M, how_long: Duration, parameter: &T) -> (u64, u64);

    /// PRIVATE
    fn test(&mut self, m: &M, parameter: &T) {
        self.bench(m, &[1u64], parameter);
    }

    /// Iterates the benchmarked function for a fixed length of time, but takes no measurements.
    /// This keeps the overall benchmark suite runtime constant-ish even when running under a
    /// profiler with an unknown amount of overhead. Since no measurements are taken, it also
    /// reduces the amount of time the execution spends in Criterion.rs code, which should help
    /// show the performance of the benchmarked code more clearly as well.
    fn profile(
        &mut self,
        measurement: &M,
        id: &BenchmarkId,
        criterion: &Criterion<M>,
        report_context: &ReportContext,
        time: Duration,
        parameter: &T,
    ) {
        criterion
            .report
            .profile(id, report_context, time.to_nanos() as f64);

        let mut profile_path = report_context.output_directory.clone();
        if (*crate::CARGO_CRITERION_CONNECTION).is_some() {
            // If connected to cargo-criterion, generate a cargo-criterion-style path.
            // This is kind of a hack.
            profile_path.push("profile");
            profile_path.push(id.as_directory_name());
        } else {
            profile_path.push(id.as_directory_name());
            profile_path.push("profile");
        }
        criterion
            .profiler
            .borrow_mut()
            .start_profiling(id.id(), &profile_path);

        let time = time.to_nanos();

        // TODO: Some profilers will show the two batches of iterations as
        // being different code-paths even though they aren't really.

        // Get the warmup time for one second
        let (wu_elapsed, wu_iters) = self.warm_up(measurement, Duration::from_secs(1), parameter);
        if wu_elapsed < time {
            // Initial guess for the mean execution time
            let met = wu_elapsed as f64 / wu_iters as f64;

            // Guess how many iterations will be required for the remaining time
            let remaining = (time - wu_elapsed) as f64;

            let iters = remaining / met;
            let iters = iters as u64;

            self.bench(measurement, &[iters], parameter);
        }

        criterion
            .profiler
            .borrow_mut()
            .stop_profiling(id.id(), &profile_path);

        criterion.report.terminated(id, report_context);
    }

    fn sample(
        &mut self,
        measurement: &M,
        id: &BenchmarkId,
        config: &BenchmarkConfig,
        criterion: &Criterion<M>,
        report_context: &ReportContext,
        parameter: &T,
    ) -> (ActualSamplingMode, Box<[f64]>, Box<[f64]>) {
        let wu = config.warm_up_time;
        let m_ns = config.measurement_time.to_nanos();

        criterion
            .report
            .warmup(id, report_context, wu.to_nanos() as f64);

        if let Some(conn) = &criterion.connection {
            conn.send(&OutgoingMessage::Warmup {
                id: id.into(),
                nanos: wu.to_nanos() as f64,
            })
            .unwrap();
        }

        let (wu_elapsed, wu_iters) = self.warm_up(measurement, wu, parameter);
        if crate::debug_enabled() {
            println!(
                "\nCompleted {} iterations in {} nanoseconds, estimated execution time is {} ns",
                wu_iters,
                wu_elapsed,
                wu_elapsed as f64 / wu_iters as f64
            );
        }

        // Initial guess for the mean execution time
        let met = wu_elapsed as f64 / wu_iters as f64;

        let n = config.sample_size as u64;

        let actual_sampling_mode = config
            .sampling_mode
            .choose_sampling_mode(met, n, m_ns as f64);

        let m_iters = actual_sampling_mode.iteration_counts(met, n, &config.measurement_time);

        let expected_ns = m_iters
            .iter()
            .copied()
            .map(|count| count as f64 * met)
            .sum();

        // Use saturating_add to handle overflow.
        let mut total_iters = 0u64;
        for count in m_iters.iter().copied() {
            total_iters = total_iters.saturating_add(count);
        }

        criterion
            .report
            .measurement_start(id, report_context, n, expected_ns, total_iters);

        if let Some(conn) = &criterion.connection {
            conn.send(&OutgoingMessage::MeasurementStart {
                id: id.into(),
                sample_count: n,
                estimate_ns: expected_ns,
                iter_count: total_iters,
            })
            .unwrap();
        }

        let m_elapsed = self.bench(measurement, &m_iters, parameter);

        let m_iters_f: Vec<f64> = m_iters.iter().map(|&x| x as f64).collect();

        (
            actual_sampling_mode,
            m_iters_f.into_boxed_slice(),
            m_elapsed.into_boxed_slice(),
        )
    }
}

pub struct Function<M: Measurement, F, T>
where
    F: FnMut(&mut Bencher<'_, M>, &T),
    T: ?Sized,
{
    f: F,
    // TODO: Is there some way to remove these?
    _phantom: PhantomData<T>,
    _phamtom2: PhantomData<M>,
}
impl<M: Measurement, F, T> Function<M, F, T>
where
    F: FnMut(&mut Bencher<'_, M>, &T),
    T: ?Sized,
{
    pub fn new(f: F) -> Function<M, F, T> {
        Function {
            f,
            _phantom: PhantomData,
            _phamtom2: PhantomData,
        }
    }
}

impl<M: Measurement, F, T> Routine<M, T> for Function<M, F, T>
where
    F: FnMut(&mut Bencher<'_, M>, &T),
    T: ?Sized,
{
    fn bench(&mut self, m: &M, iters: &[u64], parameter: &T) -> Vec<f64> {
        let f = &mut self.f;

        let mut b = Bencher {
            iterated: false,
            iters: 0,
            value: m.zero(),
            measurement: m,
            elapsed_time: Duration::from_millis(0),
        };

        iters
            .iter()
            .map(|iters| {
                b.iters = *iters;
                (*f)(&mut b, parameter);
                b.assert_iterated();
                m.to_f64(&b.value)
            })
            .collect()
    }

    fn warm_up(&mut self, m: &M, how_long: Duration, parameter: &T) -> (u64, u64) {
        let f = &mut self.f;
        let mut b = Bencher {
            iterated: false,
            iters: 1,
            value: m.zero(),
            measurement: m,
            elapsed_time: Duration::from_millis(0),
        };

        let mut total_iters = 0;
        let mut elapsed_time = Duration::from_millis(0);
        loop {
            (*f)(&mut b, parameter);

            b.assert_iterated();

            total_iters += b.iters;
            elapsed_time += b.elapsed_time;
            if elapsed_time > how_long {
                return (elapsed_time.to_nanos(), total_iters);
            }

            b.iters = b.iters.wrapping_mul(2);
        }
    }
}
