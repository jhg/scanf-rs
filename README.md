# Scanf

If you know it from C, same functionality but with memory safety.

## scanf! & sscanf!

```rust
use scanf::scanf;

let mut number: u32 = 0;
let mut name: String = String::new();
if scanf!("{},{}", number, name).is_ok() {
    println!("Input is: {} and {}", number, name);
}
```

```rust
use scanf::sscanf;

let input = "5,something";
let mut number: u32 = 0;
let mut name: String = String::new();
if let Err(error) = sscanf!(input, "{},{}", number, name) {
    panic!("Error {} using sscanf!", error);
}
```

## Examples

```no_run
use scanf::scanf;

let mut product: String = String::new();
let mut price: f32 = 0.0;
println!("Insert product and price (product: price):");
if scanf!("{}: {}", product, price).is_ok() {
    println!("Price of {} is {:.2}", product, price);
}
```

```rust
use scanf::sscanf;

let input: &str = "Candy: 2.75";
let mut product: String = String::new();
let mut price: f32 = 0.0;
println!("Insert product and price (product: price):");
sscanf!(input, "{}: {}", product, price);
println!("Price of {} is {:.2}", product, price);
# assert_eq!(product, "Candy");
# assert_eq!(price, 2.75);
```

Using variable names in format strings:

```rust
use scanf::sscanf;

let input: &str = "Candy: 2.75";
let mut product: String = String::new();
let mut price: f32 = 0.0;
sscanf!(input, "{product}: {price}", product, price);
println!("Price of {} is {:.2}", product, price);
# assert_eq!(product, "Candy");
# assert_eq!(price, 2.75);
```

You can also use generic placeholders without specifying names:

```no_run
# use scanf::scanf;
let mut product: String = String::new();
let mut price: f32 = 0.0;
println!("Insert product and price (product: price):");
scanf!("{}: {}", product, price);
# println!("Price of {} is {:.2}", product, price);
```

It's also possible to use variable names in the format string (similar to `format!`):

```no_run
# use scanf::scanf;
let mut product: String = String::new();
let mut price: f32 = 0.0;
println!("Insert product and price (product: price):");
scanf!("{product}: {price}", product, price);
# println!("Price of {} is {:.2}", product, price);
```

Also escape brackets:

```rust
# use scanf::sscanf;
let input: &str = "{Candy}";
let mut product: String = String::new();
sscanf!(input, "{{{}}}", product);
assert_eq!(product, "Candy");
```

Examples has been compiled and `sscanf`'s examples also ran as tests.
If you have problems using the example code, please, [create an issue](https://github.com/jhg/scanf-rs/issues?q=is%3Aissue).
