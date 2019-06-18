use benchmark::BenchmarkConfig;
use std::time::{Duration, Instant};

use measurement::Measurement;
use program::Program;
use report::{BenchmarkId, ReportContext};
use std::marker::PhantomData;
use {Bencher, Criterion, DurationExt};

/// PRIVATE
pub trait Routine<M: Measurement, T> {
    fn start(&mut self, parameter: &T) -> Option<Program>;

    /// PRIVATE
    fn bench(&mut self, m: &M, p: &mut Option<Program>, iters: &[u64], parameter: &T) -> Vec<f64>;
    /// PRIVATE
    fn warm_up(
        &mut self,
        m: &M,
        p: &mut Option<Program>,
        how_long: Duration,
        parameter: &T,
    ) -> (u64, u64);

    /// PRIVATE
    fn test(&mut self, m: &M, parameter: &T) {
        let mut p = self.start(parameter);
        self.bench(m, &mut p, &[1u64], parameter);
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

        let time = time.to_nanos();
        let mut p = self.start(parameter);

        // Get the warmup time for one second
        let (wu_elapsed, wu_iters) =
            self.warm_up(measurement, &mut p, Duration::from_secs(1), parameter);
        if wu_elapsed >= time {
            return;
        }

        // Initial guess for the mean execution time
        let met = wu_elapsed as f64 / wu_iters as f64;

        // Guess how many iterations will be required for the remaining time
        let remaining = (time - wu_elapsed) as f64;

        let iters = remaining / met;
        let iters = iters as u64;

        self.bench(measurement, &mut p, &[iters], parameter);

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

        let mut p = self.start(parameter);

        let (wu_elapsed, wu_iters) = self.warm_up(measurement, &mut p, wu, parameter);

        // Initial guess for the mean execution time
        let met = wu_elapsed as f64 / wu_iters as f64;

        let n = config.sample_size as u64;
        // Solve: [d + 2*d + 3*d + ... + n*d] * met = m_ns
        let total_runs = n * (n + 1) / 2;
        let d = (m_ns as f64 / met / total_runs as f64).ceil() as u64;

        let m_iters = (1..(n + 1) as u64).map(|a| a * d).collect::<Vec<u64>>();

        let m_ns = total_runs as f64 * d as f64 * met;
        criterion
            .report
            .measurement_start(id, report_context, n, m_ns, m_iters.iter().sum());
        let m_elapsed = self.bench(measurement, &mut p, &m_iters, parameter);

        let m_iters_f: Vec<f64> = m_iters.iter().map(|&x| x as f64).collect();

        (m_iters_f.into_boxed_slice(), m_elapsed.into_boxed_slice())
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
    fn start(&mut self, _: &T) -> Option<Program> {
        None
    }

    fn bench(&mut self, m: &M, _: &mut Option<Program>, iters: &[u64], parameter: &T) -> Vec<f64> {
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

    fn warm_up(
        &mut self,
        m: &M,
        _: &mut Option<Program>,
        how_long: Duration,
        parameter: &T,
    ) -> (u64, u64) {
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
