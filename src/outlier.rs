use sample::Sample;

pub struct Outliers {
    high_mild: uint,
    high_severe: uint,
    low_mild: uint,
    low_severe: uint,
    sample_size: uint,
    total: uint,
}

impl Outliers {
    // classify outliers using the boxplot method
    // see http://en.wikipedia.org/wiki/Boxplot for more information
    pub fn new(sample: &Sample) -> Outliers {
        let (q1, _, q3) = sample.quartiles();
        let iqr = q3 - q1;

        let lost = q1 - 3.0 * iqr;  // Low severe outlier threshold
        let lomt = q1 - 1.5 * iqr;  // Low mild outlier threshold
        let himt = q3 + 1.5 * iqr;  // High mild outlier threshold
        let hist = q3 + 3.0 * iqr;  // High severe outlier threshold

        let (mut los, mut lom, mut him, mut his) = (0, 0, 0, 0);

        for &value in sample.data().iter() {
            if value < lost {
                los += 1;
            } else if value < lomt {
                lom += 1;
            } else if value > hist {
                his += 1;
            } else if value > himt {
                him += 1;
            }
        }

        Outliers {
            high_mild: him,
            high_severe: his,
            low_mild: lom,
            low_severe: los,
            sample_size: sample.len(),
            total: him + his + lom + los,
        }
    }

    pub fn report(&self) {
        if self.total == 0 {
            return
        }

        let percent = |n: uint| { 100.0 * n as f64 / self.sample_size as f64 };

        println!("> found {} outliers among {} measurements ({:.2}%)",
                 self.total,
                 self.sample_size,
                 percent(self.total));

        let print = |n: uint, class| {
            if n != 0 {
                println!("  > {} ({:.2}%) {}", n, percent(n), class);
            }
        };

        print(self.low_severe, "low severe");
        print(self.low_mild, "low mild");
        print(self.high_mild, "high mild");
        print(self.high_severe, "high severe");
    }
}
