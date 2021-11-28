use criterion::{black_box, criterion_group, criterion_main, Criterion};

use scanf::sscanf;

fn sscanf_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-2000");
    group.sample_size(2000);

    let input = black_box("Candy -> 2.75");
    group.bench_function("sscanf string & f64", |b| {
        b.iter(|| {
            let product: String;
            let price: f64;
            sscanf!(input, "{} -> {}", product, price);
            black_box(product);
            black_box(price);
        })
    });

    group.finish();
}

criterion_group!(benches, sscanf_benchmark);
criterion_main!(benches);
