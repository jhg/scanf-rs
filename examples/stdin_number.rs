use scanf::scanf;

pub fn main() {
    let mut number = 0;
    scanf!("{number}").unwrap();
    println!("Input is: {}", number);
}
