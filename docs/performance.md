# Performance review

The benchmarks measure generated parsing code at runtime. Format tokenization and macro
expansion happen at compile time and are not included. These results do not measure
compilation speed or terminal I/O latency.

## Running the benchmarks

Run from a quiet machine, using the same Rust version, target, features and release profile
for both revisions:

```sh
cargo bench --bench benchmark -- --save-baseline before
# Apply the production change, keeping the benchmark harness unchanged.
cargo bench --bench benchmark -- --baseline before
```

A shorter exploratory run is useful before a full comparison:

```sh
cargo bench --bench benchmark -- --sample-size 30 --warm-up-time 0.2 --measurement-time 1 --noplot
cargo bench --bench benchmark --all-features -- --test
```

Criterion keeps measurements and confidence intervals in `target/criterion`. Compare repeated
runs and their uncertainty, rather than treating every change in the point estimate as an
optimization. Benchmark default and optional features separately: `ignore-trailing-input`
changes the trailing-input case from failure to success, and `trim-whitespace` adds a retry
only when the first parse fails.

## Coverage and measurement fixes

- Inputs cross `black_box` inside each iteration, and parsed values are observed or returned
  to Criterion. Successful parses are checked with `unwrap` instead of discarding errors.
- The string-splitting baseline uses the same literal space separator as the macro and
  constructs owned strings on both sides. `split_whitespace` has different semantics.
- Scalar parsing is compared with `str::parse`, and three numeric fields are compared with
  handwritten `split_once` and `parse`. Return, named and explicit-reference forms have the
  same numeric workload.
- Error cases cover invalid numbers, missing separators and trailing input. Their timings
  include constructing and dropping the diagnostic.
- Short and 4 KiB fields cover ASCII, Unicode and multi-character separators. These cases
  include allocating and dropping the returned `String`; they are not pure search timings.
- `fscanf!` is measured with an in-memory `Cursor`. It looks at the buffered line without
  copying it, so only a line whose end is not buffered pays for an allocation. It does not
  measure disk or stdin performance. The optional trimming retry has its own case.

Older results from the unchecked benchmark harness are not directly comparable with the
corrected suite.

## Findings (2026-10-29)

The review used Rust 1.99.1 on an AArch64 Cortex-A76 environment, with the repository's
release settings (optimization level 3, LTO and one codegen unit). The exploratory baseline
used 30 samples, a 0.2-second warm-up and a one-second measurement per case.

The machine showed substantial timing variability. For example, the 95% confidence interval
for the mean of scalar `sscanf!` parsing was 18.6–36.1 ns, compared with 16.0–25.1 ns for
`str::parse`. These intervals overlap; this run cannot establish a reliable percentage of
macro overhead or support small optimization claims. It is a baseline for investigation,
not a portable performance guarantee.

The generated success path already uses `split_once` with a character pattern for
single-character separators, parses with `FromStr`, and constructs diagnostics only on
failure. Numeric-only `sscanf!` success paths do not need an owned input buffer; returned
`String` values and line-reading macros have allocation costs that must be included when
comparing workloads.

One candidate moved a constant placeholder description into the error format string at
macro-expansion time, reducing the apparent number of runtime formatting arguments.
An optimized assembly probe produced `before = after`: this compiler already folds the
constant description into the format string. The production change was discarded because
it offered no demonstrated improvement. The generated parser and its error semantics remain
unchanged.

A comparison run used 50 samples, a 0.5-second warm-up and two-second measurements for the
representative cases. Even the unchanged `str::parse` control shifted by more than 100%,
showing that the environment was not stable across runs. A statistically significant change
in this setting cannot by itself be attributed to a source change. No speedup percentage is
claimed from this review.

Further runtime changes need repeatable evidence on a quiet target machine. In particular,
replacing generic `FromStr` parsing, changing separator matching, or bypassing owned outputs
would need correctness checks as well as timings; those behaviors are part of the API.
