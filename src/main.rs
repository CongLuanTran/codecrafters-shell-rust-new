use codecrafters_shell::builtins::BUILTINS;
use codecrafters_shell::redirections::Redirections;
use std::fs::File;
use std::io::{self, stdout, ErrorKind, Read, Write};
use std::process::{exit, Command, Stdio};

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
                            let file = File::create(parts.next().unwrap()).unwrap();
                            redirs.stdout = Some(Box::new(file));
                        }
                        _ => args.push(part.to_string()),
                    }
                }
                shell_exec(&args, redirs);
            }
        }
    }
}

fn shell_exec(args: &[String], redirs: Redirections) {
    if let Some(cmd) = BUILTINS.iter().find(|b| b.name == args[0]) {
        (cmd.func)(args);
        return;
    }
    shell_launch(args, redirs);
}

fn shell_launch(args: &[String], mut redirs: Redirections) {
    if args.is_empty() {
        eprintln!("shell_launch: args is empty");
        exit(exitcode::USAGE)
    }

    let mut builder = Command::new(&args[0]);

    if redirs.stdout.is_some() {
        builder.stdout(Stdio::piped());
    }

    let program = builder.args(&args[1..]).spawn();

    match program {
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("{}: command not found", &args[0]);
            }
            _ => {
                eprintln!("Unknown Error: {}", e);
                exit(exitcode::UNAVAILABLE)
            }
        },
        Ok(mut child) => {
            if let Some(mut child_stdout) = child.stdout.take() {
                let mut buffer = Vec::new();
                child_stdout.read_to_end(&mut buffer).unwrap();
                let mut stdout = redirs.stdout.take().unwrap_or(Box::new(stdout()));
                stdout.write_all(&buffer).unwrap();
            }
            if let Err(e) = child.wait() {
                eprintln!("Unknown Error: {}", e)
            }
        }
    }
}
