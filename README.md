# Scanf

If you know it from C, same functionality but with memory safety, plus new enhanced features!

## Usage

Like Rust's `format!` macro, scanf supports automatic variable capture:

```no_run
use scanf::scanf;

let mut number: u32 = 0;
let mut name: String = String::new();
if scanf!("{number},{name}").is_ok() {
    println!("Input is: {} and {}", number, name);
}
```

```rust
use scanf::sscanf;

let input = "5,something";
let mut number: u32 = 0;
let mut name: String = String::new();

sscanf!(input, "{number},{name}").unwrap();
assert_eq!(number, 5);
assert_eq!(name, "something");
```

### Escape brackets

```rust
# use scanf::sscanf;
let input: &str = "{Candy}";
let mut product: String = String::new();
sscanf!(input, "{{{product}}}").unwrap();
assert_eq!(product, "Candy");
```

## Examples

### Enhanced approach with implicit capture

```rust
use scanf::sscanf;

let input: &str = "Candy: 2.75";
let mut product: String = String::new();
let mut price: f32 = 0.0;

sscanf!(input, "{product}: {price}").unwrap();
println!("Price of {} is {:.2}", product, price);
# assert_eq!(product, "Candy");
# assert_eq!(price, 2.75);
```

### Traditional approach

```no_run
use scanf::scanf;

let mut product: String = String::new();
let mut price: f32 = 0.0;
println!("Insert product and price (product: price):");
if scanf!("{}: {}", &mut product, &mut price).is_ok() {
    println!("Price of {} is {:.2}", product, price);
}
```

### Mixed syntax - named and anonymous placeholders

```rust
use scanf::sscanf;

let input: &str = "Alice: 25 years";
let mut name: String = String::new();
let mut age: i32 = 0;
let mut unit: String = String::new();

sscanf!(input, "{name}: {} {unit}", &mut age).unwrap();
assert_eq!(name, "Alice");
assert_eq!(age, 25);
assert_eq!(unit, "years");
```

### Traditional Syntax

```no_run
use scanf::scanf;

let mut number: u32 = 0;
let mut name: String = String::new();
if scanf!("{},{}", &mut number, &mut name).is_ok() {
    println!("Input is: {} and {}", number, name);
}
```

```rust
use scanf::sscanf;

let input = "5,something";
let mut number: u32 = 0;
let mut name: String = String::new();
if let Err(error) = sscanf!(input, "{},{}", &mut number, &mut name) {
    panic!("Error {} using sscanf!", error);
}
```

## Features

### `trim-whitespace` (disabled by default)

By default every captured value is parsed exactly as it appears in the input, so `" 42 "` is
not a valid integer. Enabling this feature makes a capture that fails to parse be retried once
with its surrounding whitespace trimmed:

```toml
scanf = { version = "2", features = ["trim-whitespace"] }
```

```text
sscanf!(" 42 ", "{number}")      // number: i32      -> 42
sscanf!("  hi  ,7", "{s},{n}")   // s: String        -> "  hi  " (kept as is)
sscanf!("1 2", "{number}")       // number: i32      -> error (only the edges are trimmed)
```

How it works and what to keep in mind:

- The macro cannot see the type of the variable, so the rule is "retry on failure", not
  "trim numbers". A `String` never fails to parse, so it always keeps its whitespace.
- Only leading and trailing whitespace is removed. Whitespace between two values is never
  interpreted: `"1 2"` into a single number is still an error.
- **Limitation:** the retry applies to every type implementing `FromStr`. A custom type whose
  `FromStr` deliberately rejects surrounding whitespace, but accepts the trimmed text, will
  accept it with this feature enabled.
- If both attempts fail, the error of the first (untrimmed) attempt is reported.
- Trailing newlines read by `scanf!` are always removed, with or without this feature.

Examples have been compiled and `sscanf`'s examples also run as tests.
If you have problems using the example code, please [create an issue](https://github.com/jhg/scanf-rs/issues?q=is%3Aissue).
