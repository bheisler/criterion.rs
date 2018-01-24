use report::{Report, MeasurementData};
use Criterion;

use handlebars::Handlebars;
use fs;

#[derive(Serialize)]
struct Context {
    test: String,
}

pub struct Html {
    handlebars: Handlebars,
}
impl Html {
    pub fn new() -> Html {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("report", include_str!("benchmark_report.html.handlebars")).unwrap();
        Html{ handlebars }
    }
}
impl Report for Html {
    fn benchmark_start(&self, _: &str, _: &Criterion) {}
    fn warmup(&self, _: &str, _: &Criterion, _: f64) {}
    fn analysis(&self, _: &str, _: &Criterion) {}
    fn measurement_start(&self, _: &str, _: &Criterion, _: u64, _: f64, _: u64) {}
    fn measurement_complete(&self, id: &str, criterion: &Criterion, measurements: &MeasurementData) {
        let context = Context {
            test: "test string".to_owned()
        };

        let text = self.handlebars.render("report", &context).unwrap();
        fs::save_string(text,
            &format!("{}/{}/index.html", criterion.output_directory, id)).unwrap();
    }
}
