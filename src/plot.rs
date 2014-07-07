use simplot::Figure;
use simplot::plottype::Lines;
use test::stats::Stats;

use fs;
use math;

// XXX should the size of the image be configurable?
pub static PNG_SIZE: (uint, uint) = (1366, 768);

pub fn kde(sample: &[f64], dir: &Path) {
    fs::mkdirp(dir);

    // TODO add legend/caption
    let (xs, ys) = math::kde(sample);
    let ys = ys.as_slice();
    let vertical = [ys.min(), ys.max()];
    let mean = sample.mean();
    let median = sample.median();
    let mean = [mean, mean];
    let median = [median, median];

    Figure::new().
        set_output_file(dir.join("kde.png")).
        set_title("Probability Density Function").
        set_ylabel("Density (a.u.)").
        set_xlabel("Time (ns)").
        set_size(PNG_SIZE).
        plot(Lines, xs.iter(), ys.iter(), []).
        plot(Lines, mean.iter(), vertical.iter(), []).
        plot(Lines, median.iter(), vertical.iter(), []).
        draw();
}
