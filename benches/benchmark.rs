use std::{error::Error, hint::black_box, str::FromStr};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

use scanf::{fscanf, sscanf};

const U16_NUMBERS_SEPARATED_BY_COMMAS: [&str; 5] = [
    "1,2,3,4,5,6,7,8,9,0",
    "161,162,163,164,165,166,167,168,169,161",
    "65535,65535,65535,65535,65535,65535,65535,65535,65535,65535",
    "45641,13422,24233,34124,55423,45236,23457,24578,06239,00000",
    "56981,52353,13123,14241,24445,03466,42357,24658,63469,18760",
];

fn sscanf_10_same_elements_of<T: Default + FromStr>(input: &str) -> (T, T, T, T, T, T, T, T, T, T)
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
        black_box(input),
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
    (
        number0, number1, number2, number3, number4, number5, number6, number7, number8, number9,
    )
}

fn sscanf_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("split-benchmark");
    let input = black_box("Candy 2.75");
    let mut product = String::new();
    let mut price = String::new();
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("Split with sscanf", |b| {
        b.iter(|| {
            sscanf!(black_box(input), "{} {}", &mut product, &mut price).unwrap();
            black_box((&product, &price));
        })
    });
    group.bench_function("Split with str::split_once", |b| {
        b.iter(|| {
            let (first, second) = black_box(input).split_once(' ').unwrap();
            product = first.to_string();
            price = second.to_string();
            black_box((&product, &price));
        })
    });
    group.finish();

    let mut group = c.benchmark_group("throughput-benchmark");
    for (i, &input) in U16_NUMBERS_SEPARATED_BY_COMMAS.iter().enumerate() {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            format!("Sscanf u16 as u16 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u16>(black_box(input))),
        );
        group.bench_with_input(
            format!("Sscanf u16 as u32 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u32>(black_box(input))),
        );
        group.bench_with_input(
            format!("Sscanf u16 as u64 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u64>(black_box(input))),
        );
        group.bench_with_input(
            format!("Sscanf u16 as u128 separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<u128>(black_box(input))),
        );
        group.bench_with_input(
            format!("Sscanf u16 as String separated by commas {}", i),
            input,
            |b, input| b.iter(|| sscanf_10_same_elements_of::<String>(black_box(input))),
        );
    }
    group.finish();

    let input = black_box("-5");
    c.bench_function("Sscanf i32", |b| {
        b.iter(|| {
            let mut first_number: i32 = 0;
            sscanf!(black_box(input), "{}", &mut first_number).unwrap();
            black_box(first_number);
        })
    });

    let input = black_box("-5");
    c.bench_function("Sscanf i64", |b| {
        b.iter(|| {
            let mut first_number: i64 = 0;
            sscanf!(black_box(input), "{}", &mut first_number).unwrap();
            black_box(first_number);
        })
    });

    let input = black_box("5");
    c.bench_function("Sscanf u32", |b| {
        b.iter(|| {
            let mut first_number: u32 = 0;
            sscanf!(black_box(input), "{}", &mut first_number).unwrap();
            black_box(first_number);
        })
    });

    let input = black_box("5");
    c.bench_function("Sscanf u64", |b| {
        b.iter(|| {
            let mut first_number: u64 = 0;
            sscanf!(black_box(input), "{}", &mut first_number).unwrap();
            black_box(first_number);
        })
    });

    let input = black_box("2.5");
    c.bench_function("Sscanf f32", |b| {
        b.iter(|| {
            let mut first_number: f32 = 0.0;
            sscanf!(black_box(input), "{}", &mut first_number).unwrap();
            black_box(first_number);
        })
    });

    let input = black_box("2.5");
    c.bench_function("Sscanf f64", |b| {
        b.iter(|| {
            let mut first_number: f64 = 0.0;
            sscanf!(black_box(input), "{}", &mut first_number).unwrap();
            black_box(first_number);
        })
    });

    let input = black_box("Candy");
    c.bench_function("Sscanf string", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(black_box(input), "{}", &mut product).unwrap();
            black_box(product);
        })
    });

    let input = black_box("{Candy}");
    c.bench_function("Sscanf string with brackets", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(black_box(input), "{}", &mut product).unwrap();
            black_box(product);
        })
    });

    let input = black_box("{Candy}");
    c.bench_function("Sscanf string with brackets ignored", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            sscanf!(black_box(input), "{{{}}}", &mut product).unwrap();
            black_box(product);
        })
    });

    let input = black_box("Candy -> 2.75");
    c.bench_function("Sscanf string & f64", |b| {
        b.iter(|| {
            let mut product: String = String::new();
            let mut price: f64 = 0.0;
            sscanf!(black_box(input), "{} -> {}", &mut product, &mut price).unwrap();
            black_box(product);
            black_box(price);
        })
    });

    let input = black_box("5 -> 2.5");
    c.bench_function("Sscanf u32 & f64", |b| {
        b.iter(|| {
            let mut first_number: u32 = 0;
            let mut second_number: f64 = 0.0;
            sscanf!(
                black_box(input),
                "{} -> {}",
                &mut first_number,
                &mut second_number
            )
            .unwrap();
            black_box(first_number);
            black_box(second_number);
        })
    });

    let input = black_box("id=42;name=Candy;price=2.75");
    c.bench_function("Sscanf literal text around 3 placeholders", |b| {
        b.iter(|| {
            let mut id: u32 = 0;
            let mut name: String = String::new();
            let mut price: f64 = 0.0;
            sscanf!(
                black_box(input),
                "id={};name={};price={}",
                &mut id,
                &mut name,
                &mut price
            )
            .unwrap();
            black_box((id, &name, price));
        })
    });

    let input = black_box("12:34:56");
    c.bench_function("Sscanf colon separated u8 x3", |b| {
        b.iter(|| {
            let (mut h, mut m, mut s) = (0u8, 0u8, 0u8);
            sscanf!(black_box(input), "{}:{}:{}", &mut h, &mut m, &mut s).unwrap();
            black_box((h, m, s));
        })
    });
}

criterion_group!(benches, sscanf_benchmark, representative_benchmarks);
criterion_main!(benches);

// Keep equivalent workloads together: each consumes a runtime input and returns its result.
fn representative_benchmarks(c: &mut Criterion) {
    let mut scalar = c.benchmark_group("scalar");
    scalar.bench_function("sscanf_return_u32", |b| {
        b.iter(|| {
            let value: u32 = sscanf!(black_box("123456"), "{}").unwrap();
            value
        })
    });
    scalar.bench_function("str_parse_u32", |b| {
        b.iter(|| black_box("123456").parse::<u32>().unwrap())
    });
    scalar.finish();

    let mut record = c.benchmark_group("record");
    record.bench_function("return", |b| {
        b.iter(|| {
            let values: (u32, u32, u32) = sscanf!(black_box("12:34:56"), "{}:{}:{}").unwrap();
            values
        })
    });
    record.bench_function("named", |b| {
        b.iter(|| {
            let (mut h, mut m, mut s) = (0u32, 0u32, 0u32);
            sscanf!(black_box("12:34:56"), "{h}:{m}:{s}").unwrap();
            (h, m, s)
        })
    });
    record.bench_function("explicit", |b| {
        b.iter(|| {
            let (mut h, mut m, mut s) = (0u32, 0u32, 0u32);
            sscanf!(black_box("12:34:56"), "{}:{}:{}", &mut h, &mut m, &mut s).unwrap();
            (h, m, s)
        })
    });
    record.bench_function("split_once_parse", |b| {
        b.iter(|| {
            let (h, rest) = black_box("12:34:56").split_once(':').unwrap();
            let (m, s) = rest.split_once(':').unwrap();
            (
                h.parse::<u32>().unwrap(),
                m.parse::<u32>().unwrap(),
                s.parse::<u32>().unwrap(),
            )
        })
    });
    record.finish();

    let mut errors = c.benchmark_group("errors");
    errors.bench_function("invalid_number", |b| {
        b.iter(|| {
            let result: std::io::Result<(u32, u32)> = sscanf!(black_box("12:invalid"), "{}:{}");
            result.unwrap_err()
        })
    });
    errors.bench_function("missing_separator", |b| {
        b.iter(|| {
            let result: std::io::Result<(u32, u32)> = sscanf!(black_box("123456"), "{}:{}");
            result.unwrap_err()
        })
    });
    errors.bench_function("trailing_input", |b| {
        b.iter(|| {
            let result: std::io::Result<u32> = sscanf!(black_box("42 units extra"), "{} units");
            // With ignore-trailing-input enabled this is a successful parse.
            result
        })
    });
    errors.finish();

    let mut separators = c.benchmark_group("separators");
    for length in [16, 4096] {
        for (name, separator) in [("ascii", ":"), ("unicode", "→"), ("multi", " -> ")] {
            let input = format!("{}{separator}42", "x".repeat(length));
            separators.throughput(Throughput::Bytes(input.len() as u64));
            separators.bench_function(format!("{name}/{length}"), |b| {
                b.iter(|| {
                    let result: (String, u32) = match name {
                        "ascii" => sscanf!(black_box(&input), "{}:{}").unwrap(),
                        "unicode" => sscanf!(black_box(&input), "{}→{}").unwrap(),
                        _ => sscanf!(black_box(&input), "{} -> {}").unwrap(),
                    };
                    result
                })
            });
        }
    }
    separators.finish();

    c.bench_function("reader/fscanf_cursor", |b| {
        b.iter(|| {
            let mut reader = std::io::Cursor::new(black_box("12:34:56\n"));
            let values: (u32, u32, u32) = fscanf!(reader, "{}:{}:{}").unwrap();
            values
        })
    });
    #[cfg(feature = "trim-whitespace")]
    c.bench_function("features/trim_retry", |b| {
        b.iter(|| {
            let value: u32 = sscanf!(black_box(" 123456 "), "{}").unwrap();
            value
        })
    });
}
