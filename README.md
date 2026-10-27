# Scanf

`scanf!` and `sscanf!` for Rust, in the style of C's `scanf`: parse text according to a format
string, the inverse of `print!` and `format!`. The crate contains no `unsafe` code.

## Usage

`sscanf!` parses a string and `scanf!` reads one line from stdin, with the same format syntax.
There are two ways to get the values out, and both work with either macro.

### Returning the values

Leave the placeholders empty and the values are returned. The types come from your
annotations, and you don't have to declare any variable first:

```rust
use scanf::sscanf;

let age: u32 = sscanf!("age=30", "age={}").unwrap();
assert_eq!(age, 30);

// Several values come back as a tuple
let (name, score): (String, f64) = sscanf!("Ana: 9.5", "{}: {}").unwrap();
assert_eq!(name, "Ana");
assert_eq!(score, 9.5);
```

The same from stdin:

```no_run
use scanf::scanf;

let (number, name): (u32, String) = scanf!("{},{}").unwrap();
println!("Input is: {number} and {name}");
```

### Named placeholders

Like Rust's `format!`, a `{name}` placeholder refers to the variable of that name, and the
parsed value is written into it. Use it when the variables already exist, or when the names
make the format easier to read:

```rust
use scanf::sscanf;

let input: &str = "Candy: 2.75";
let mut product: String = String::new();
let mut price: f32 = 0.0;

sscanf!(input, "{product}: {price}").unwrap();
println!("Price of {product} is {price:.2}");
# assert_eq!(product, "Candy");
# assert_eq!(price, 2.75);
```

```no_run
use scanf::scanf;

let mut number: u32 = 0;
let mut name: String = String::new();
if scanf!("{number},{name}").is_ok() {
    println!("Input is: {number} and {name}");
}
```

Both ways can be combined. Named placeholders write into their variable, and the empty ones
are returned:

```rust
use scanf::sscanf;

let mut id: u32 = 0;
let (name, score): (String, f64) = sscanf!("7|Ana|9.5", "{id}|{}|{}").unwrap();
assert_eq!((id, name.as_str(), score), (7, "Ana", 9.5));
```

### Escaping braces

```rust
# use scanf::sscanf;
let input: &str = "{Candy}";
let mut product: String = String::new();
sscanf!(input, "{{{product}}}").unwrap();
assert_eq!(product, "Candy");
```

### Errors

Both macros return an `io::Result`. Input that does not match the format is an
`InvalidInput` error, and `scanf!` fails with `UnexpectedEof` when stdin has no more lines.
With named placeholders, values parsed before a failure are already written, so check the
result before using the variables. Mistakes in the format string itself are compile errors.

```rust
use scanf::sscanf;

let result: std::io::Result<i32> = sscanf!("abc", "{}");
assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
```

## Limitations

- Placeholders cannot be adjacent (`{}{}`): there would be no way to tell where the first
  one ends. Put text between them.
- A placeholder followed by text parses up to the first occurrence of that text, with no
  backtracking.
- Parsed types must implement `FromStr`.
- Input is a single line. `scanf!` removes one trailing line ending (`\n`, `\r\n` or a lone
  `\r`). It reads up to the next `\n`, so a stream that separates lines with `\r` alone arrives
  as one long line.
- Input after a trailing literal in the format is not checked: `"{} units"` accepts
  `"5 units extra"`.
- Format strings are limited to 10,000 bytes, 256 tokens (text segments and placeholders)
  and 128-character placeholder names.

```rust
use scanf::sscanf;

// The first separator wins, and text after the trailing literal is ignored.
let (key, value): (String, String) = sscanf!("a=b=c units, extra", "{}={} units").unwrap();
assert_eq!((key.as_str(), value.as_str()), ("a", "b=c"));
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
- The line ending read by `scanf!` is always removed, with or without this feature.

If you have problems using the macros, please [create an issue](https://github.com/jhg/scanf-rs/issues?q=is%3Aissue).
