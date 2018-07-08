use csv::Writer;
use error::Result;
use report::{BenchmarkId, MeasurementData, Report, ReportContext};
use std::io::Write;

#[derive(Serialize)]
struct CsvRow<'a> {
    group: &'a str,
    function: Option<&'a str>,
    value: Option<&'a str>,
    sample_time_nanos: f64,
    iteration_count: u64,
}

struct CsvReportWriter<W: Write> {
    writer: Writer<W>,
}
impl<W: Write> CsvReportWriter<W> {
    fn write_data(&mut self, id: &BenchmarkId, data: &MeasurementData) -> Result<()> {
        for (count, time) in data.iter_counts
            .as_slice()
            .iter()
            .zip(data.sample_times.as_slice())
        {
            let row = CsvRow {
                group: id.group_id.as_str(),
                function: id.function_id.as_ref().map(String::as_str),
                value: id.value_str.as_ref().map(String::as_str),
                sample_time_nanos: *time,
                iteration_count: (*count) as u64,
            };
            self.writer.serialize(row)?;
        }
        Ok(())
    }
}

pub struct FileCsvReport;
impl FileCsvReport {
    fn write_file(
        &self,
        path: String,
        id: &BenchmarkId,
        measurements: &MeasurementData,
    ) -> Result<()> {
        let writer = Writer::from_path(path)?;
        let mut writer = CsvReportWriter { writer };
        writer.write_data(id, measurements)?;
        Ok(())
    }
}

impl Report for FileCsvReport {
    fn measurement_complete(
        &self,
        id: &BenchmarkId,
        context: &ReportContext,
        measurements: &MeasurementData,
    ) {
        let path = format!(
            "{}/{}/new/raw.csv",
            context.output_directory,
            id.as_directory_name()
        );
        log_if_err!(self.write_file(path, id, measurements));
    }
}
