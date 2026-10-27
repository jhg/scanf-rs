use scanf::scanf;

// Reads "name: age" from stdin into existing variables.
// Try it with: echo "Ana: 30" | cargo run --example stdin_named
fn main() {
    let mut name = String::new();
    let mut age: u32 = 0;
    scanf!("{name}: {age}").unwrap();
    println!("{name} is {age} years old");
}
