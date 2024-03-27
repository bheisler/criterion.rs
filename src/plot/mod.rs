mod gnuplot_backend;
#[cfg(feature = "plotters")]
mod plotters_backend;

pub(crate) use gnuplot_backend::Gnuplot;
#[cfg(feature = "plotters")]
pub(crate) use plotters_backend::PlottersBackend;

use crate::estimate::Statistic;
use crate::measurement::ValueFormatter;
use crate::report::{BenchmarkId, ComparisonData, MeasurementData, ReportContext, ValueType};
use crate::Throughput;
use std::path::PathBuf;

const REPORT_STATS: [Statistic; 7] = [
    Statistic::Typical,
    Statistic::Slope,
    Statistic::Mean,
    Statistic::Median,
    Statistic::MedianAbsDev,
    Statistic::MedianAbsDev,
    Statistic::StdDev,
];
const CHANGE_STATS: [Statistic; 2] = [Statistic::Mean, Statistic::Median];
#[derive(Clone, Copy)]
pub(crate) struct PlotContext<'a> {
    pub(crate) id: &'a BenchmarkId,
    pub(crate) context: &'a ReportContext,
    pub(crate) size: Option<(usize, usize)>,
    pub(crate) is_thumbnail: bool,
}

impl<'a> PlotContext<'a> {
    pub fn size(mut self, s: Option<criterion_plot::Size>) -> PlotContext<'a> {
        if let Some(s) = s {
            self.size = Some((s.0, s.1));
        }
        self
    }

    pub fn thumbnail(mut self, value: bool) -> PlotContext<'a> {
        self.is_thumbnail = value;
        self
    }

    pub fn line_comparison_path(&self) -> PathBuf {
        let mut path = self.context.output_directory.clone();
        path.push(self.id.as_directory_name());
        path.push("report");
        path.push("lines.svg");
        path
    }

    pub fn line_throughput_comparison_path(&self) -> PathBuf {
        let mut path = self.context.output_directory.clone();
        path.push(self.id.as_directory_name());
        path.push("report");
        path.push("lines_throughput.svg");
        path
    }

    pub fn violin_path(&self) -> PathBuf {
        let mut path = self.context.output_directory.clone();
        path.push(self.id.as_directory_name());
        path.push("report");
        path.push("violin.svg");
        path
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PlotData<'a> {
    pub(crate) formatter: &'a dyn ValueFormatter,
    pub(crate) measurements: &'a MeasurementData<'a>,
    pub(crate) comparison: Option<&'a ComparisonData>,
}

impl<'a> PlotData<'a> {
    pub fn comparison(mut self, comp: &'a ComparisonData) -> PlotData<'a> {
        self.comparison = Some(comp);
        self
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LinePlotConfig {
    label: &'static str,
    scale: fn(&dyn ValueFormatter, &BenchmarkId, f64, &BenchmarkId, &mut [f64]) -> &'static str,
    path: fn(&PlotContext<'_>) -> PathBuf,
}

impl LinePlotConfig {
    pub fn time() -> Self {
        Self {
            label: "time",
            scale: |formatter, _, max, _, vals| formatter.scale_values(max, vals),
            path: |ctx| ctx.line_comparison_path(),
        }
    }

    pub fn throughput() -> Self {
        Self {
            label: "throughput",
            scale: |formatter, max_id, max, id, vals| {
                // Scale values to be in line with max_id throughput
                let from = id
                    .throughput
                    .as_ref()
                    .expect("Throughput chart expects throughput to be defined");
                let to = max_id.throughput.as_ref().unwrap();

                let (from_bytes, to_bytes) = match (from, to) {
                    (Throughput::Bytes(from), Throughput::Bytes(to)) => (from, to),
                    (Throughput::BytesDecimal(from), Throughput::BytesDecimal(to)) => (from, to),
                    (Throughput::Elements(from), Throughput::Elements(to)) => (from, to),
                    _ => unreachable!("throughput types expected to be equal"),
                };

                let mul = *to_bytes as f64 / *from_bytes as f64;

                for val in vals.iter_mut() {
                    *val *= mul;
                }

                formatter.scale_throughputs(max, to, vals)
            },
            path: |ctx| ctx.line_throughput_comparison_path(),
        }
    }
}

pub(crate) trait Plotter {
    fn pdf(&mut self, ctx: PlotContext<'_>, data: PlotData<'_>);

    fn regression(&mut self, ctx: PlotContext<'_>, data: PlotData<'_>);

    fn iteration_times(&mut self, ctx: PlotContext<'_>, data: PlotData<'_>);

    fn abs_distributions(&mut self, ctx: PlotContext<'_>, data: PlotData<'_>);

    fn rel_distributions(&mut self, ctx: PlotContext<'_>, data: PlotData<'_>);

    fn line_comparison(
        &mut self,
        line_config: LinePlotConfig,
        ctx: PlotContext<'_>,
        formatter: &dyn ValueFormatter,
        all_curves: &[&(&BenchmarkId, Vec<f64>)],
        value_type: ValueType,
    );

    fn violin(
        &mut self,
        ctx: PlotContext<'_>,
        formatter: &dyn ValueFormatter,
        all_curves: &[&(&BenchmarkId, Vec<f64>)],
    );

    fn t_test(&mut self, ctx: PlotContext<'_>, data: PlotData<'_>);

    fn wait(&mut self);
}
