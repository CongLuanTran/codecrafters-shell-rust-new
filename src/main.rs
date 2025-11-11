#[allow(unused_imports)]
use std::collections::HashMap;
use std::io::{self, Write};

fn main() {
    let mut builtins: HashMap<&str, fn(&str)> = HashMap::new();
    builtins.insert("exit", exit);
    builtins.insert("echo", echo);

    loop {
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        for line in input.lines() {
            if let Some((cmd, args)) = line.split_once(" ") {
                if let Some(func) = builtins.get(cmd) {
                    func(args);
                } else {
                    println!("{}: command not found", cmd);
                }
            } else {
                println!("{}: command not found", line);
            }
        }
    }
}

fn exit(args: &str) {
    let code = args.parse::<i32>().unwrap_or_default();
    std::process::exit(code)
}

fn echo(args: &str) {
    println!("{}", args)
}
