#![allow(unused_must_use)]
#![allow(clippy::needless_return)]

use std::{any::Any, error::Error, str::FromStr};

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};

use scanf::sscanf;

const U16_NUMBERS_SEPARATED_BY_COMMAS: [&str; 5] = [
    "1,2,3,4,5,6,7,8,9,0",
    "161,162,163,164,165,166,167,168,169,161",
    "65535,65535,65535,65535,65535,65535,65535,65535,65535,65535",
    "45641,13422,24233,34124,55423,45236,23457,24578,06239,00000",
    "56981,52353,13123,14241,24445,03466,42357,24658,63469,18760",
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
        &mut number0,
        &mut number1,
        &mut number2,
        &mut number3,
        &mut number4,
        &mut number5,
        &mut number6,
        &mut number7,
        &mut number8,
        &mut number9
    )
    .unwrap();
    return (
        number0, number1, number2, number3, number4, number5, number6, number7, number8, number9,
    );
}

const INPUT_FORMATS: [&str; 4] = ["", "{}", "{},{}", "{string},{u64}"];

fn sscanf_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("split-benchmark");
    let input = black_box("Candy 2.75");
    let mut product = String::new();
    let mut price = String::new();
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("Split with sscanf", |b| {
        b.iter(|| {
            sscanf!(input, "{} {}", &mut product, &mut price);
        })
    });
    group.bench_function("Split with str::split", |b| {
        b.iter(|| {
            let mut split = input.split_whitespace();
            product = split.next().unwrap().to_string();
            price = split.next().unwrap().to_string();
        })
    });
    group.finish();

    let mut group = c.benchmark_group("input-format-parse-benchmark");
    for input_format in INPUT_FORMATS {
        group.throughput(Throughput::Bytes(input_format.len() as u64));
        group.bench_with_input(
            format!("Parse input format {:?}", input_format),
            input_format,
            |b, input_format| {
                b.iter(|| {
                    let input_parser = scanf::format::InputFormatParser::new(input_format).unwrap();
                    black_box(input_parser);
                })
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("throughput-benchmark");
    for (i, &input) in U16_NUMBERS_SEPARATED_BY_COMMAS.iter().enumerate() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            format!("Sscanf u16 as u16 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u16>(input)),
        );
        group.bench_with_input(
            format!("Sscanf u16 as u32 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u32>(input)),
        );
        group.bench_with_input(
            format!("Sscanf u16 as u64 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u64>(input)),
        );
        group.bench_with_input(
            format!("Sscanf u16 as u128 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u128>(input)),
        );
        group.bench_with_input(
            format!("Sscanf u16 as String separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<String>(input)),
        );
    }
    group.finish();

    let input = black_box("-5");
    c.bench_function("Sscanf i32", |b| {
        b.iter(|| {
            let mut first_number: i32 = 0;
            sscanf!(input, "{}", &mut first_number);
            black_box(first_number);
        })
    });

    let input = black_box("-5");
    c.bench_function("Sscanf i64", |b| {
        b.iter(|| {
            let mut first_number: i64 = 0;
            sscanf!(input, "{}", &mut first_number);
            black_box(first_number);
        })
    });

    let input = black_box("5");
    c.bench_function("Sscanf u32", |b| {
        b.iter(|| {
            let mut first_number: u32 = 0;
            sscanf!(input, "{}", &mut first_number);
            black_box(first_number);
        })
    });

    let input = black_box("5");
    c.bench_function("Sscanf u64", |b| {
        b.iter(|| {
            let mut first_number: u64 = 0;
            sscanf!(input, "{}", &mut first_number);
            black_box(first_number);
        })
    });

    let input = black_box("2.5");
    c.bench_function("Sscanf f32", |b| {
        b.iter(|| {
            let mut first_number: f32 = 0.0;
            sscanf!(input, "{}", &mut first_number);
            black_box(first_number);
        })
    });

    let input = black_box("2.5");
    c.bench_function("Sscanf f64", |b| {
        b.iter(|| {
            let mut first_number: f64 = 0.0;
            sscanf!(input, "{}", &mut first_number);
            black_box(first_number);
        })
    });

    let input = black_box("Candy");
    c.bench_function("Sscanf string", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(input, "{}", &mut product);
            black_box(product);
        })
    });

    let input = black_box("{Candy}");
    c.bench_function("Sscanf string with brackets", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(input, "{}", &mut product);
            black_box(product);
        })
    });

    let input = black_box("{Candy}");
    c.bench_function("Sscanf string with brackets ignored", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(input, "{{{}}}", &mut product);
            black_box(product);
        })
    });

    let input = black_box("Candy -> 2.75");
    c.bench_function("Sscanf string & f64", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            let mut price: f64 = 0.0;
            sscanf!(input, "{} -> {}", &mut product, &mut price);
            black_box(product);
            black_box(price);
        })
    });

    let input = black_box("5 -> 2.5");
    c.bench_function("Sscanf u32 & f64", |b| {
        b.iter(|| {
            let mut first_number: u32 = 0;
            let mut second_number: f64 = 0.0;
            sscanf!(input, "{} -> {}", &mut first_number, &mut second_number);
            black_box(first_number);
            black_box(second_number);
        })
    });
}

criterion_group!(benches, sscanf_benchmark);
criterion_main!(benches);
