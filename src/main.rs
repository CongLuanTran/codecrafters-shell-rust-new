use codecrafters_shell::builtins::BUILTINS;
use codecrafters_shell::redirections::{ErrorTarget, OutputTarget, Redirections};
use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;
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
            if let Ok(parts) = shellwords::split(line) {
                let mut parts = parts.iter();
                let mut args = Vec::new();
                let mut redirs = Redirections::new();
                while let Some(part) = parts.next() {
                    match part.as_str() {
                        ">" | "1>" => {
                            let file = PathBuf::from(parts.next().unwrap());
                            redirs.stdout = OutputTarget::File {
                                path: file,
                                handle: None,
                                append: false,
                            }
                        }
                        ">>" | "1>>" => {
                            let file = PathBuf::from(parts.next().unwrap());
                            redirs.stdout = OutputTarget::File {
                                path: file,
                                handle: None,
                                append: true,
                            }
                        }
                        "2>" => {
                            let file = PathBuf::from(parts.next().unwrap());
                            redirs.stderr = ErrorTarget::File {
                                path: file,
                                handle: None,
                                append: false,
                            }
                        }
                        "2>>" => {
                            let file = PathBuf::from(parts.next().unwrap());
                            redirs.stderr = ErrorTarget::File {
                                path: file,
                                handle: None,
                                append: true,
                            }
                        }
                        _ => args.push(part.to_string()),
                    }
                }
                shell_exec(&args, &mut redirs);
            }
        }
    }
}

fn shell_exec(args: &[String], redirs: &mut Redirections) {
    if args.is_empty() {
        eprintln!("shell_exec: args is empty");
        exit(exitcode::UNAVAILABLE)
    }

    if let Some(cmd) = BUILTINS.iter().find(|b| b.name == args[0]) {
        (cmd.func)(args, redirs);
        return;
    }
    shell_launch(args, redirs);
}

fn shell_launch(args: &[String], redirs: &mut Redirections) {
    if args.is_empty() {
        eprintln!("shell_launch: args is empty");
        exit(exitcode::UNAVAILABLE)
    }
    let mut exec = Command::new(&args[0]);
    let program = exec
        .args(&args[1..])
        .stdout(redirs.stdout.to_stdio())
        .stderr(redirs.stderr.to_stdio());
    if let Err(e) = program.status() {
        match e.kind() {
            ErrorKind::NotFound => eprintln!("{}: command not found", args[0]),
            _ => {
                eprintln!("shell_launch: {}", e);
                exit(exitcode::UNAVAILABLE)
            }
        }
    };
}
