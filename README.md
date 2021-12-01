# Scanf

If you now it from C, nothing to tell you about. Same functionality but with memory safety.

```rust
let index: u32 = 0;
let name: String = String::new();
scanf!("{},{}", number, name); // Parse input line like "5,something"
```

Look more [examples in the documentation](https://docs.rs/scanf/latest/scanf/#examples).
