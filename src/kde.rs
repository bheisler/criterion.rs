use stats::kde::Kde;

use test::stats::Stats;

pub fn sweep(sample: &[f64], npoints: uint) -> (Vec<f64>, Vec<f64>) {
    let x_min = sample.min();
    let x_max = sample.max();

    let kde = Kde::new(sample);
    let h = kde.bandwidth();

    let xy = kde.sweep((x_min - 3. * h, x_max + 3. * h), npoints);

    let mut x = Vec::with_capacity(npoints);
    let mut y = Vec::with_capacity(npoints);

    for &(a, b) in xy.iter() {
        x.push(a);
        y.push(b);
    }

    (x, y)
}
