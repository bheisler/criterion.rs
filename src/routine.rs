use benchmark::BenchmarkConfig;
use std::time::{Duration, Instant};

use program::Program;
use report::{BenchmarkId, ReportContext};
use std::marker::PhantomData;
use {Bencher, Criterion, DurationExt};

/// PRIVATE
pub trait Routine<T> {
    fn start(&mut self, parameter: &T) -> Option<Program>;

    /// PRIVATE
    fn bench(&mut self, m: &mut Option<Program>, iters: &[u64], parameter: &T) -> Vec<f64>;
    /// PRIVATE
    fn warm_up(&mut self, m: &mut Option<Program>, how_long: Duration, parameter: &T)
        -> (u64, u64);

    /// PRIVATE
    fn test(&mut self, parameter: &T) {
        let mut m = self.start(parameter);
        self.bench(&mut m, &[1u64], parameter);
    }

    fn sample(
        &mut self,
        id: &BenchmarkId,
        config: &BenchmarkConfig,
        criterion: &Criterion,
        report_context: &ReportContext,
        parameter: &T,
    ) -> (Box<[f64]>, Box<[f64]>) {
        let wu = config.warm_up_time;
        let m_ns = config.measurement_time.to_nanos();

        criterion
            .report
            .warmup(id, report_context, wu.to_nanos() as f64);

        let mut m = self.start(parameter);

        let (wu_elapsed, wu_iters) = self.warm_up(&mut m, wu, parameter);

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
        let m_elapsed = self.bench(&mut m, &m_iters, parameter);

        let m_iters_f: Vec<f64> = m_iters.iter().map(|&x| x as f64).collect();

        (m_iters_f.into_boxed_slice(), m_elapsed.into_boxed_slice())
    }
}

pub struct Function<F, T>
where
    F: FnMut(&mut Bencher, &T),
{
    f: F,
    _phantom: PhantomData<T>,
}
impl<F, T> Function<F, T>
where
    F: FnMut(&mut Bencher, &T),
{
    pub fn new(f: F) -> Function<F, T> {
        Function {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<F, T> Routine<T> for Function<F, T>
where
    F: FnMut(&mut Bencher, &T),
{
    fn start(&mut self, _: &T) -> Option<Program> {
        None
    }

    fn bench(&mut self, _: &mut Option<Program>, iters: &[u64], parameter: &T) -> Vec<f64> {
        let f = &mut self.f;

        let mut b = Bencher {
            iterated: false,
            iters: 0,
            elapsed: Duration::from_secs(0),
        };

        iters
            .iter()
            .map(|iters| {
                b.iters = *iters;
                (*f)(&mut b, parameter);
                b.assert_iterated();
                b.elapsed.to_nanos() as f64
            })
            .collect()
    }

    fn warm_up(
        &mut self,
        _: &mut Option<Program>,
        how_long: Duration,
        parameter: &T,
    ) -> (u64, u64) {
        let f = &mut self.f;
        let mut b = Bencher {
            iterated: false,
            iters: 1,
            elapsed: Duration::from_secs(0),
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
