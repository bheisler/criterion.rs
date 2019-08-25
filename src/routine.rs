use crate::benchmark::BenchmarkConfig;
use crate::measurement::Measurement;
use crate::report::{BenchmarkId, ReportContext};
use crate::{Bencher, Criterion, DurationExt};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// PRIVATE
pub trait Routine<M: Measurement, T> {
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

        let profile_path = PathBuf::from(format!(
            "{}/{}/profile",
            report_context.output_directory,
            id.as_directory_name()
        ));
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
    ) -> (Box<[f64]>, Box<[f64]>) {
        let wu = config.warm_up_time;
        let m_ns = config.measurement_time.to_nanos();

        criterion
            .report
            .warmup(id, report_context, wu.to_nanos() as f64);

        let (wu_elapsed, wu_iters) = self.warm_up(measurement, wu, parameter);

        // Initial guess for the mean execution time
        let met = wu_elapsed as f64 / wu_iters as f64;

        let n = config.sample_size as u64;
        // Solve: [d + 2*d + 3*d + ... + n*d] * met = m_ns
        let total_runs = n * (n + 1) / 2;
        let d = (m_ns as f64 / met / total_runs as f64).ceil() as u64;
        let expected_ns = total_runs as f64 * d as f64 * met;

        if d == 1 {
            let recommended_sample_size = recommend_sample_size(m_ns as f64, met);
            let actual_time = Duration::from_nanos(expected_ns as u64);
            println!("\nWarning: Unable to complete {} samples in {:.1?}. You may wish to increase target time to {:.1?} or reduce sample count to {}",
                n, config.measurement_time, actual_time, recommended_sample_size);
        }

        let m_iters = (1..(n + 1) as u64).map(|a| a * d).collect::<Vec<u64>>();

        criterion.report.measurement_start(
            id,
            report_context,
            n,
            expected_ns,
            m_iters.iter().sum(),
        );
        let m_elapsed = self.bench(measurement, &m_iters, parameter);

        let m_iters_f: Vec<f64> = m_iters.iter().map(|&x| x as f64).collect();

        (m_iters_f.into_boxed_slice(), m_elapsed.into_boxed_slice())
    }
}

fn recommend_sample_size(target_time: f64, met: f64) -> u64 {
    // Some math shows that n(n+1)/2 * d * met = target_time. d = 1, so it can be ignored.
    // This leaves n(n+1) = (2*target_time)/met, or n^2 + n - (2*target_time)/met = 0
    // Which can be solved with the quadratic formula. Since A and B are constant 1,
    // this simplifies to sample_size = (-1 +- sqrt(1 - 4C))/2, where C = (2*target_time)/met.
    // We don't care about the negative solution. Experimentation shows that this actually tends to
    // result in twice the desired execution time (probably because of the ceil used to calculate
    // d) so instead I use c = target_time/met.
    let c = target_time / met;
    let sample_size = (-1.0 + (4.0 * c).sqrt()) / 2.0;
    let sample_size = sample_size as u64;

    // Round down to the nearest 10 to give a margin and avoid excessive precision
    let sample_size = (sample_size / 10) * 10;

    // Clamp it to be at least 10, since criterion.rs doesn't allow sample sizes smaller than 10.
    if sample_size < 10 {
        10
    } else {
        sample_size
    }
}

pub struct Function<M: Measurement, F, T>
where
    F: FnMut(&mut Bencher<M>, &T),
{
    f: F,
    // TODO: Is there some way to remove these?
    _phantom: PhantomData<T>,
    _phamtom2: PhantomData<M>,
}
impl<M: Measurement, F, T> Function<M, F, T>
where
    F: FnMut(&mut Bencher<M>, &T),
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
    F: FnMut(&mut Bencher<M>, &T),
{
    fn bench(&mut self, m: &M, iters: &[u64], parameter: &T) -> Vec<f64> {
        let f = &mut self.f;

        let mut b = Bencher {
            iterated: false,
            iters: 0,
            value: m.zero(),
            measurement: m,
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
        };

        let mut total_iters = 0;
        let start = Instant::now();
        loop {
            (*f)(&mut b, parameter);

            b.assert_iterated();

            total_iters += b.iters;
            let elapsed = start.elapsed();
            if elapsed > how_long {
                return (elapsed.to_nanos(), total_iters);
            }

            b.iters *= 2;
        }
    }
}
