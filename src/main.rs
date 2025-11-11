#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // let builtins = [""];
    let mut input = String::new();
    // TODO: Uncomment the code below to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    for line in input.lines() {
        println!("{}: command not found", line);
    }
}
