extern crate criterion;

pub use criterion::Criterion;
pub use criterion::black_box;

pub struct Bencher<'a> {
    pub bytes: u64,
    pub bencher: &'a mut ::criterion::Bencher,
}
impl<'a> Bencher<'a> {
    pub fn iter<T, F>(&mut self, inner: F)
        where F: FnMut() -> T
    {
        self.bencher.iter(inner);
    }
}

#[macro_export]
macro_rules! benchmark_group {
    ($group_name:ident, $($function:path),+) => {
        pub fn $group_name() {
            use $crate::Criterion;
            let mut criterion: Criterion = Criterion::default().configure_from_args();

            $(
                criterion.bench_function(stringify!($function), |b| {
                    let mut wrapped = $crate::Bencher {
                        bytes: 0,
                        bencher: b,
                    };

                    $function(&mut wrapped);
                });
            )+
        }
    };
    ($group_name:ident, $($function:path,)+) => {
        benchmark_group!($group_name, $($function),+);
    };
}

#[macro_export]
macro_rules! benchmark_main {
    ($($group_name:path),+) => {
        fn main() {
            $(
                $group_name();
            )+

            $crate::Criterion::default()
                .configure_from_args()
                .final_summary();
        }
    };
    ($($group_name:path,)+) => {
        benchmark_main!($($group_name),+);
    };
}