use scanf::sscanf;

// The two recommended forms side by side. Run it with: cargo run --example parse_line
fn main() {
    // Returned values: the types come from the annotation.
    let (name, score): (String, f64) = sscanf!("Ana: 9.5", "{}: {}").unwrap();
    println!("{name} scored {score}");

    // Named placeholders: each one assigns to the variable of the same name.
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    sscanf!("x=10, y=20", "x={x}, y={y}").unwrap();
    println!("point is ({x}, {y})");

    // A failed parse is an error, never a panic.
    let result: std::io::Result<u8> = sscanf!("300", "{}");
    println!("{result:?}");
}
