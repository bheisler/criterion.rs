use simplot::Figure;

use criterion::PNG_SIZE;
use fs;
use math;

pub fn kde(sample: &[f64], dir: &Path) {
    fs::mkdirp(dir);

    // TODO plot mean median
    let (xs, ys) = math::kde(sample);
    Figure::new().
        set_output_file(dir.join("kde.png")).
        set_size(PNG_SIZE).
        plot(xs.iter(), ys.iter()).
        draw();
}
