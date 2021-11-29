use criterion::{black_box, criterion_group, criterion_main, Criterion};

use scanf::sscanf;

fn sscanf_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-2500");
    group.sample_size(2_500);

    let input = black_box("-5");
    group.bench_function("sscanf i32", |b| {
        b.iter(|| {
            let first_number: i32;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("-5");
    group.bench_function("sscanf i64", |b| {
        b.iter(|| {
            let first_number: i64;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("5");
    group.bench_function("sscanf u32", |b| {
        b.iter(|| {
            let first_number: u32;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("5");
    group.bench_function("sscanf u64", |b| {
        b.iter(|| {
            let first_number: u64;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("2.5");
    group.bench_function("sscanf f32", |b| {
        b.iter(|| {
            let first_number: f32;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("2.5");
    group.bench_function("sscanf f64", |b| {
        b.iter(|| {
            let first_number: f64;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("Candy");
    group.bench_function("sscanf string", |b| {
        b.iter(|| {
            let product: String;
            sscanf!(input, "{}", product);
            black_box(product);
        })
    });

    let input = black_box("{Candy}");
    group.bench_function("sscanf string with brackets", |b| {
        b.iter(|| {
            let product: String;
            sscanf!(input, "{}", product);
            black_box(product);
        })
    });

    let input = black_box("{Candy}");
    group.bench_function("sscanf string with brackets ignored", |b| {
        b.iter(|| {
            let product: String;
            sscanf!(input, "{{{}}}", product);
            black_box(product);
        })
    });

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

    let input = black_box("5 -> 2.5");
    group.bench_function("sscanf u32 & f64", |b| {
        b.iter(|| {
            let first_number: u32;
            let second_number: f64;
            sscanf!(input, "{} -> {}", first_number, second_number);
            black_box(first_number);
            black_box(second_number);
        })
    });

    group.finish();
}

criterion_group!(benches, sscanf_benchmark);
criterion_main!(benches);
