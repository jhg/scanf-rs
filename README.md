# Scanf

If you know it from C, same functionality but with memory safety, plus new enhanced features!

## scanf! & sscanf! - Now with Implicit Variable Capture!

### Traditional Syntax (Still Supported)
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

### New Implicit Variable Capture (v2.0+)
Like Rust's `format!` macro, scanf now supports automatic variable capture:

```rust
use scanf::sscanf;

let input = "5,something";
let mut number: u32 = 0;
let mut name: String = String::new();
// Variables captured automatically by name!
sscanf!(input, "{number},{name}").unwrap();
assert_eq!(number, 5);
assert_eq!(name, "something");
```

## Examples

### Traditional approach:
```no_run
use scanf::scanf;

let mut product: String = String::new();
let mut price: f32 = 0.0;
println!("Insert product and price (product: price):");
if scanf!("{}: {}", &mut product, &mut price).is_ok() {
    println!("Price of {} is {:.2}", product, price);
}
```

### Enhanced approach with implicit capture:
```rust
use scanf::sscanf;

let input: &str = "Candy: 2.75";
let mut product: String = String::new();
let mut price: f32 = 0.0;
// Automatic variable capture - no need to pass arguments!
sscanf!(input, "{product}: {price}").unwrap();
println!("Price of {} is {:.2}", product, price);
# assert_eq!(product, "Candy");
# assert_eq!(price, 2.75);
```

### Mixed syntax - named and anonymous placeholders:
```rust
use scanf::sscanf;

let input: &str = "Alice: 25 years";
let mut name: String = String::new();
let mut age: i32 = 0;
let mut unit: String = String::new();
// Mix automatic capture and explicit arguments
sscanf!(input, "{name}: {} {unit}", &mut age).unwrap();
assert_eq!(name, "Alice");
assert_eq!(age, 25);
assert_eq!(unit, "years");
```

### Escape brackets:
```rust
# use scanf::sscanf;
let input: &str = "{Candy}";
let mut product: String = String::new();
sscanf!(input, "{{{product}}}").unwrap();
assert_eq!(product, "Candy");
```

Examples have been compiled and `sscanf`'s examples also run as tests.
If you have problems using the example code, please [create an issue](https://github.com/jhg/scanf-rs/issues?q=is%3Aissue).
