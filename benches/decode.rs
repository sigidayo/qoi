use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use qoi::decode::decode;

macro_rules! bench {
    ($c:expr, $case:expr) => {
        let label = format!("decode_{}", $case);
        let data = include_bytes!(concat!("../data/", $case, ".qoi"));

        $c.bench_function(&label, |b| {
            b.iter(|| {
                decode(black_box(data)).unwrap();
            });
        });
    };
}

fn benchmark_decode(c: &mut Criterion) {
    bench!(c, "dice");
    bench!(c, "edgecase");
    bench!(c, "kodim10");
    bench!(c, "kodim23");
    bench!(c, "qoi_logo");
    bench!(c, "testcard");
    bench!(c, "testcard_rgba");
    bench!(c, "wikipedia_008");
}

criterion_group!(benches, benchmark_decode);
criterion_main!(benches);
