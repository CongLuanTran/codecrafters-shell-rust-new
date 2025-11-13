use codecrafters_shell::builtins::BUILTINS;
use std::io::{self, ErrorKind, Write};
use std::process::{exit, Command};

fn main() {
    loop {
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            break;
        };
        for line in input.lines() {
            if let Ok(args) = shellwords::split(line) {
                shell_exec(&args);
            }
        }
    }
}

fn shell_exec(args: &[String]) {
    if let Some(cmd) = BUILTINS.iter().find(|b| b.name == args[0]) {
        (cmd.func)(args);
        return;
    }
    shell_launch(args);
}

fn shell_launch(args: &[String]) {
    if args.is_empty() {
        eprintln!("shell_launch: args is empty");
        exit(exitcode::USAGE)
    }

    match Command::new(&args[0]).args(&args[1..]).status() {
        Ok(_) => {}
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("{}: command not found", &args[0]);
            }
            _ => {
                eprintln!("Unknown Error: {}", e);
                exit(exitcode::UNAVAILABLE)
            }
        },
    }
}
