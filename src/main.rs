use codecrafters_shell::builtins::BUILTINS;
use codecrafters_shell::readline::create_readline;
use codecrafters_shell::redirections::{
    parse_redirections, InputSource, OutputTarget, Redirections,
};
use rustyline::error::ReadlineError;
use rustyline::history::History;
use std::env::var_os;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{self, ErrorKind};
use std::process::{exit, Child, Command};

fn main() -> rustyline::Result<()> {
    let mut rl = create_readline()?;
    if let Some(histfile) = var_os("HISTFILE") {
        if rl.load_history(&histfile).is_err() {
            eprintln!("History file not found")
        }
    }
    let mut history_pointer = 0;
    let mut history_exit_pointer = rl.history().len();

    // Main loop
    loop {
        // Readline
        let readline = rl.readline("$ ");

        // Handle Ctrl-D and Ctrl-C
        let mut input = String::new();
        match readline {
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Input Error: {}", err)
            }
            Ok(lines) => {
                history_exit_pointer += 1;
                input = lines.to_string();
            }
        }

        if let Ok(parts) = shellwords::split(&input) {
            let mut pipeline: Vec<(Vec<String>, Redirections)> = Vec::new();
            let pipes = parts.split(|x| *x == "|").peekable();
            for pipe in pipes {
                pipeline.push(parse_redirections(pipe));
            }
            let mut it = pipeline.iter_mut().peekable();
            let mut prev_stdin = None;
            while let Some((_, redirs)) = it.next() {
                if let Some(prev) = prev_stdin {
                    redirs.stdin = prev;
                    prev_stdin = None;
                }
                if it.peek().is_some() && matches!(redirs.stdout, OutputTarget::Stdout(_)) {
                    let (reader, writer) = os_pipe::pipe()?;
                    redirs.stdout = OutputTarget::Pipe(writer);
                    prev_stdin = Some(InputSource::Pipe(reader));
                }
            }

            let mut children = vec![];
            for (args, mut redirs) in pipeline {
                if args[0] == "exit" {
                    let code = args.get(1).and_then(|n| n.parse().ok()).unwrap_or_default();
                    if let Some(histfile) = var_os("HISTFILE") {
                        if !histfile.is_empty() {
                            let mut file = OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(histfile)?;
                            for line in rl.history().iter().skip(history_exit_pointer) {
                                writeln!(file, "{}", line)?;
                            }
                        }
                    }
                    exit(code);
                }
                if args[0] == "history" {
                    let mut it = args.iter().peekable();
                    if it.len() == 1 {
                        it.next().unwrap();
                        for (i, entry) in rl.history().iter().enumerate() {
                            writeln!(redirs.stdout, "\t{} {}", i + 1, entry)?
                        }
                    } else if it.len() == 2 {
                        it.next().unwrap();
                        if let Some(thing) = it.next() {
                            if let Ok(num) = thing.parse::<usize>() {
                                let skip = rl.history().len().saturating_sub(num);
                                for (i, entry) in rl.history().iter().enumerate().skip(skip) {
                                    writeln!(redirs.stdout, "\t{} {}", i + 1, entry)?
                                }
                            }
                        }
                    } else if it.len() == 3 {
                        it.next().unwrap();
                        if let Some(thing) = it.next() {
                            if let Some(path) = it.next() {
                                match thing.as_str() {
                                    "-r" => rl.load_history(path)?,
                                    "-w" => {
                                        let mut file = OpenOptions::new()
                                            .write(true)
                                            .truncate(true)
                                            .create(true)
                                            .open(path)?;
                                        for entry in rl.history() {
                                            writeln!(file, "{}", entry)?
                                        }
                                    }

                                    "-a" => {
                                        let mut file = OpenOptions::new()
                                            .append(true)
                                            .create(true)
                                            .open(path)?;
                                        for line in rl.history().iter().skip(history_pointer) {
                                            writeln!(file, "{}", line)?;
                                        }
                                        history_pointer = rl.history().len()
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    continue;
                }

                let res = shell_exec(&args, &mut redirs);
                match res {
                    Ok(child) => {
                        drop(redirs);
                        if let Some(child) = child {
                            children.push(child);
                        }
                    }
                    Err(e) => match e.kind() {
                        ErrorKind::NotFound => {
                            writeln!(redirs.stderr, "{}: command not found", args[0])?;
                        }
                        _ => {
                            eprintln!("Unknown Error: {}", e);
                        }
                    },
                }
            }

            for mut child in children {
                child.wait()?;
            }
        }
    }

    Ok(())
}

fn shell_exec(args: &[String], redirs: &mut Redirections) -> io::Result<Option<Child>> {
    if args.is_empty() {
        eprintln!("shell_exec: args is empty");
        exit(exitcode::UNAVAILABLE)
    }

    if let Some(cmd) = BUILTINS.iter().find(|b| b.name == args[0]) {
        (cmd.func)(args, redirs)?;
        Ok(None)
    } else {
        Ok(Some(shell_launch(args, redirs)?))
    }
}

fn shell_launch(args: &[String], redirs: &mut Redirections) -> io::Result<Child> {
    if args.is_empty() {
        eprintln!("shell_launch: args is empty");
        exit(exitcode::UNAVAILABLE)
    }
    let mut exec = Command::new(&args[0]);

    let child = exec
        .args(&args[1..])
        .stdin(redirs.stdin.to_stdio()?)
        .stdout(redirs.stdout.to_stdio()?)
        .stderr(redirs.stderr.to_stdio()?)
        .spawn()?;

    Ok(child)
}
