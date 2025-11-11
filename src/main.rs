#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // let builtins = [""];
    loop {
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        for line in input.lines() {
            println!("{}: command not found", line);
        }
    }
}
