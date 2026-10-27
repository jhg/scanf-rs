use scanf::scanf;

// Reads one number from stdin. Try it with: echo 42 | cargo run --example stdin_number
fn main() {
    let number: i32 = scanf!("{}").unwrap();
    println!("Input is: {number}");
}
