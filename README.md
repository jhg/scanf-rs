# Scanf

If you know it from C, same functionality but with memory safety.

```rust
let mut number: u32 = 0;
let mut name: String = String::new();
if scanf!("{},{}", number, name).is_ok() {
    println!("Input is: {} and {}", number, name);
}
```

```rust
let input = "5,something";
let mut number: u32 = 0;
let mut name: String = String::new();
if let Err(error) = sscanf!(input, "{},{}", number, name) {
    panic!("Error {} using sscanf!", error);
}
```

Look more [examples in the documentation](https://docs.rs/scanf/latest/scanf/#examples).
