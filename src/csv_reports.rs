use csv::Writer;
use failure;
use fs;
use report::{BenchmarkId, MeasurementData, Report, ReportContext};
use std::path::Path;

pub struct CsvReport {}

impl CsvReport {
    pub fn new() -> CsvReport {
        CsvReport {}
    }
}

impl Report for CsvReport {
    fn report_init(&self, report_context: &ReportContext) {
        let path = Path::new(&report_context.output_directory).join("benchmark-raw.csv");
        try_else_return!(fs::save_string("group,function,parameter,elapsed\n", &path));
    }

    fn benchmark_start(&self, _: &BenchmarkId, _: &ReportContext) {}
    fn warmup(&self, _: &BenchmarkId, _: &ReportContext, _: f64) {}
    fn terminated(&self, _: &BenchmarkId, _: &ReportContext) {}
    fn analysis(&self, _: &BenchmarkId, _: &ReportContext) {}
    fn measurement_start(&self, _: &BenchmarkId, _: &ReportContext, _: u64, _: f64, _: u64) {}
    fn final_summary(&self, _: &ReportContext) {}
    fn summarize(&self, _: &ReportContext, _: &[BenchmarkId]) {}
    fn measurement_complete(
        &self,
        id: &BenchmarkId,
        report_context: &ReportContext,
        measurements: &MeasurementData,
    ) {
        let path = Path::new(&report_context.output_directory).join("benchmark-raw.csv");
        let output = try_else_return!(fs::append(path));
        let mut wtr = Writer::from_writer(output);

        for avg_time in measurements.avg_times.as_slice() {
            let write_res = wtr.write_record(&[
                id.group_id.as_str(),
                &id.function_id.as_ref().map_or("", String::as_str),
                &id.value_str.as_ref().map_or("", String::as_str),
                &avg_time.to_string(),
            ]);

            try_else_return!(write_res.map_err(failure::Error::from));
        }

        try_else_return!(wtr.flush().map_err(failure::Error::from));
    }
}
