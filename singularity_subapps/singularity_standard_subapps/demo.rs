fn main() {
    println!("Hi!");

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();

    println!("Read: {}", buffer);
}
