# Scanf

`scanf!` and `sscanf!` for Rust, in the style of C's `scanf`: parse text according to a format string, the inverse of `print!` and `format!`.

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

### Reading from any reader

`fscanf!` is `sscanf!` for readers. It takes any `BufRead` first, borrowed mutably like the
writer of `write!`, and each call consumes one line:

```rust
use scanf::fscanf;
use std::io::Cursor;

let mut input = Cursor::new("Ana: 9.5\nBob: 7\n");

let (name, score): (String, f64) = fscanf!(input, "{}: {}").unwrap();
assert_eq!((name.as_str(), score), ("Ana", 9.5));

let mut name = String::new();
let mut score: f64 = 0.0;
fscanf!(input, "{name}: {score}").unwrap();
assert_eq!((name.as_str(), score), ("Bob", 7.0));
```

A call always consumes the whole line, even if the format only uses part of it or the line
does not match; the next call starts on the next line. When the reader has no more lines the
call fails with `UnexpectedEof`, and a line that is not valid UTF-8 fails with `InvalidData`.
Errors from the reader itself are passed on unchanged.

`scanf!` locks stdin on every call. If your code already holds a `stdin().lock()`, a call to
`scanf!` on the same thread waits forever, because that lock is not reentrant. Read from the
lock you hold instead:

```no_run
use scanf::fscanf;

let mut input = std::io::stdin().lock();
let (a, b): (i32, i32) = fscanf!(input, "{} {}").unwrap();
```

### Errors

Both macros return an `io::Result`. Input that does not match the format is an
`InvalidInput` error, and `scanf!` fails with `UnexpectedEof` when stdin has no more lines.
Nothing is written to your variables unless every placeholder parses, so a failed call
leaves them as they were. Mistakes in the format string itself are compile errors.

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
- When the format ends with literal text, the input must end there too: `"{} units"` rejects
  `"5 units extra"`. The `ignore-trailing-input` feature relaxes this.
- Format strings are limited to 10,000 bytes, 256 tokens (text segments and placeholders)
  and 128-character placeholder names.

```rust
use scanf::sscanf;

// The first separator wins.
let (key, value): (String, String) = sscanf!("a=b=c units", "{}={} units").unwrap();
assert_eq!((key.as_str(), value.as_str()), ("a", "b=c"));
```

## Features

Both features are disabled by default. Enable either or both in `Cargo.toml`:

```toml
scanf = { version = "2", features = ["ignore-trailing-input", "trim-whitespace"] }
```

### `ignore-trailing-input`

Allows extra input after the format's final literal text: `"{} units"` accepts
`"5 units extra"` instead of returning `InvalidInput`. All literal text must still match.
If the format ends with a placeholder, it still captures the rest of the input.

### `trim-whitespace`

Retries a failed parse with leading and trailing whitespace removed. Successful parses are
unchanged, so `String` values keep their whitespace:

```text
sscanf!(" 42 ", "{number}")      // number: i32      -> 42
sscanf!("  hi  ,7", "{s},{n}")   // s: String        -> "  hi  " (kept as is)
sscanf!("1 2", "{number}")       // number: i32      -> error (only the edges are trimmed)
```

This applies to all `FromStr` types, including custom types that normally reject surrounding
whitespace. If both attempts fail, the original error is returned. `scanf!` removes the line
ending regardless of this feature.

If you have problems using the macros, please [create an issue](https://github.com/jhg/scanf-rs/issues?q=is%3Aissue).
