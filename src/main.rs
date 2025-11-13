use codecrafters_shell::builtins::BUILTINS;
use codecrafters_shell::redirections::{ErrorTarget, OutputTarget, Redirections};
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, ErrorKind, Result, Write};
use std::process::{exit, Command};

fn main() -> Result<()> {
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
                let mut openner = OpenOptions::new();
                let options = openner.create(true).write(true);
                while let Some(part) = parts.next() {
                    match part.as_str() {
                        ">" | "1>" => {
                            let path = parts.next().unwrap();
                            let file = options.truncate(true).open(path).unwrap();
                            redirs.stdout = OutputTarget::File(file);
                        }
                        ">>" | "1>>" => {
                            let path = parts.next().unwrap();
                            let file = options.truncate(false).append(true).open(path).unwrap();
                            redirs.stdout = OutputTarget::File(file);
                        }
                        "2>" => {
                            let path = parts.next().unwrap();
                            let file = options.truncate(true).open(path).unwrap();
                            redirs.stderr = ErrorTarget::File(file);
                        }
                        "2>>" => {
                            let path = parts.next().unwrap();
                            let file = options.truncate(false).append(true).open(path).unwrap();
                            redirs.stderr = ErrorTarget::File(file);
                        }
                        _ => args.push(part.to_string()),
                    }
                }
                shell_exec(&args, &mut redirs)?;
            }
        }
    }

    Ok(())
}

fn shell_exec(args: &[String], redirs: &mut Redirections) -> Result<()> {
    if args.is_empty() {
        eprintln!("shell_exec: args is empty");
        exit(exitcode::UNAVAILABLE)
    }

    let res = if let Some(cmd) = BUILTINS.iter().find(|b| b.name == args[0]) {
        (cmd.func)(args, redirs)
    } else {
        shell_launch(args, redirs)
    };

    if let Err(e) = res {
        match e.kind() {
            ErrorKind::NotFound => {
                writeln!(redirs.stderr, "{}: command not found", args[0])?;
            }
            _ => {
                eprintln!("Unknown Error: {}", e);
            }
        }
    }
    Ok(())
}

fn shell_launch(args: &[String], redirs: &mut Redirections) -> Result<()> {
    if args.is_empty() {
        eprintln!("shell_launch: args is empty");
        exit(exitcode::UNAVAILABLE)
    }
    let mut exec = Command::new(&args[0]);
    let mut child = exec
        .args(&args[1..])
        .stdout(redirs.stdout.to_stdio())
        .stderr(redirs.stderr.to_stdio())
        .spawn()?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            writeln!(redirs.stdout, "{}", line)?;
        }
    }

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line?;
            writeln!(redirs.stderr, "{}", line)?;
        }
    }

    child.wait()?;
    Ok(())
}
