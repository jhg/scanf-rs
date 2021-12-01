# Scanf

If you now it from C, nothing to tell you about. Same functionality but with memory safety.

```rust
let index: u32 = 0;
let name: String = String::new();
if scanf!("{},{}", number, name).is_ok() {
    println!("Input is: {} and {}", number, name);
}
```

```rust
let input = "5,something";
let index: u32 = 0;
let name: String = String::new();
if let Err(error) = sscanf!(input, "{},{}", number, name) {
    panic!("Error {} using sscang!", error);
}
```

Look more [examples in the documentation](https://docs.rs/scanf/latest/scanf/#examples).
