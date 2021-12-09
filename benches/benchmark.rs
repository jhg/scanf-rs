#![allow(unused_must_use)]

use std::{any::Any, error::Error, str::FromStr};

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use scanf::sscanf;

const TEN_U16_NUMBERS_SEPARATED_BY_COMMAS: [&str; 10] = [
    "1,2,3,4,5,6,7,8,9,0",
    "11,12,13,14,15,16,17,18,19,20",
    "161,162,163,164,165,166,167,168,169,161",
    "12453,22325,35645,47872,57834,63276,63876,09283,45673,04132",
    "65535,65535,65535,65535,65535,65535,65535,65535,65535,65535",
    "18456,24574,45673,36754,35675,64673,45547,23458,46549,34620",
    "45641,13422,24233,34124,55423,45236,23457,24578,06239,00000",
    "56336,43532,45324,45345,34534,34789,32474,35853,26994,43530",
    "56981,52353,13123,14241,24445,03466,42357,24658,63469,18760",
    "53669,53456,45613,14241,53435,23426,23567,35248,34539,34534",
];

fn sscanf_10_same_elements_of<T: Default + FromStr + Any>(
    input: &str,
) -> (T, T, T, T, T, T, T, T, T, T)
where
    <T as FromStr>::Err: Error + Send + Sync,
{
    let (
        mut number0,
        mut number1,
        mut number2,
        mut number3,
        mut number4,
        mut number5,
        mut number6,
        mut number7,
        mut number8,
        mut number9,
    ) = (
        T::default(),
        T::default(),
        T::default(),
        T::default(),
        T::default(),
        T::default(),
        T::default(),
        T::default(),
        T::default(),
        T::default(),
    );
    sscanf!(
        input,
        "{},{},{},{},{},{},{},{},{},{}",
        number0,
        number1,
        number2,
        number3,
        number4,
        number5,
        number6,
        number7,
        number8,
        number9
    )
    .unwrap();
    return (
        number0, number1, number2, number3, number4, number5, number6, number7, number8, number9,
    );
}

fn sscanf_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput-benchmark");
    for (i, &input) in TEN_U16_NUMBERS_SEPARATED_BY_COMMAS.iter().enumerate() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            format!("sscanf 10 u16 as u16 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u16>(input)),
        );
        group.bench_with_input(
            format!("sscanf 10 u16 as u32 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u32>(input)),
        );
        group.bench_with_input(
            format!("sscanf 10 u16 as u64 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u64>(input)),
        );
        group.bench_with_input(
            format!("sscanf 10 u16 as u128 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u128>(input)),
        );
        group.bench_with_input(
            format!("sscanf 10 u16 as String separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<String>(input)),
        );
    }
    group.finish();

    let input = black_box("-5");
    c.bench_function("sscanf i32", |b| {
        b.iter(|| {
            let mut first_number: i32 = 0;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("-5");
    c.bench_function("sscanf i64", |b| {
        b.iter(|| {
            let mut first_number: i64 = 0;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("5");
    c.bench_function("sscanf u32", |b| {
        b.iter(|| {
            let mut first_number: u32 = 0;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("5");
    c.bench_function("sscanf u64", |b| {
        b.iter(|| {
            let mut first_number: u64 = 0;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("2.5");
    c.bench_function("sscanf f32", |b| {
        b.iter(|| {
            let mut first_number: f32 = 0.0;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("2.5");
    c.bench_function("sscanf f64", |b| {
        b.iter(|| {
            let mut first_number: f64 = 0.0;
            sscanf!(input, "{}", first_number);
            black_box(first_number);
        })
    });

    let input = black_box("Candy");
    c.bench_function("sscanf string", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(input, "{}", product);
            black_box(product);
        })
    });

    let input = black_box("{Candy}");
    c.bench_function("sscanf string with brackets", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(input, "{}", product);
            black_box(product);
        })
    });

    let input = black_box("{Candy}");
    c.bench_function("sscanf string with brackets ignored", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(input, "{{{}}}", product);
            black_box(product);
        })
    });

    let input = black_box("Candy -> 2.75");
    c.bench_function("sscanf string & f64", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            let mut price: f64 = 0.0;
            sscanf!(input, "{} -> {}", product, price);
            black_box(product);
            black_box(price);
        })
    });

    let input = black_box("5 -> 2.5");
    c.bench_function("sscanf u32 & f64", |b| {
        b.iter(|| {
            let mut first_number: u32 = 0;
            let mut second_number: f64 = 0.0;
            sscanf!(input, "{} -> {}", first_number, second_number);
            black_box(first_number);
            black_box(second_number);
        })
    });
}

criterion_group!(benches, sscanf_benchmark);
criterion_main!(benches);
