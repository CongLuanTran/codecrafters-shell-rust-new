use codecrafters_shell::builtins::{list_path, BUILTINS};
use codecrafters_shell::redirections::{ErrorTarget, InputSource, OutputTarget, Redirections};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{
    Cmd, CompletionType, Config, EditMode, Editor, Helper, Hinter, KeyEvent, Validator,
};
use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::process::{exit, Command};
use trie_rs::{Trie, TrieBuilder};

#[derive(Helper, Hinter, Validator)]
struct MyHelper {
    trie: Trie<u8>,
    #[rustyline(Completer)]
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

impl Highlighter for MyHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let prefix = &line[..pos];
        let targets = self
            .trie
            .predictive_search(prefix)
            .map(|s: String| Pair {
                display: s.clone(),
                replacement: s + " ",
            })
            .collect();
        Ok((0, targets))
    }
}

fn main() -> rustyline::Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .bell_style(rustyline::config::BellStyle::Audible)
        .build();

    // Trie for completion search
    let mut builder = TrieBuilder::new();
    // Gather builtins command name
    for builtin in BUILTINS {
        builder.push(builtin.name);
    }
    // Gather names of executables on path
    for path in list_path()? {
        let name = path.file_name().unwrap().to_str().unwrap();
        builder.push(name);
    }
    let trie = builder.build();

    // Create Rustyline helper
    let h = MyHelper {
        trie,
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter::new(),
        validator: MatchingBracketValidator::new(),
    };

    // Set up editor
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);

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
                rl.add_history_entry(lines.as_str())?;
                input = lines.to_string()
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

            for (args, mut redirs) in pipeline {
                shell_exec(&args, &mut redirs)?;
            }
        }
    }

    Ok(())
}

// Handle redirections
fn parse_redirections(words: &[String]) -> (Vec<String>, Redirections) {
    let mut words = words.iter();
    let mut args = Vec::new();
    let mut redirs = Redirections::new();
    let mut openner = OpenOptions::new();
    let options = openner.create(true).write(true);
    while let Some(part) = words.next() {
        match part.as_str() {
            ">" | "1>" => {
                let path = words.next().unwrap();
                let file = options.truncate(true).open(path).unwrap();
                redirs.stdout = OutputTarget::File(file);
            }
            ">>" | "1>>" => {
                let path = words.next().unwrap();
                let file = options.truncate(false).append(true).open(path).unwrap();
                redirs.stdout = OutputTarget::File(file);
            }
            "2>" => {
                let path = words.next().unwrap();
                let file = options.truncate(true).open(path).unwrap();
                redirs.stderr = ErrorTarget::File(file);
            }
            "2>>" => {
                let path = words.next().unwrap();
                let file = options.truncate(false).append(true).open(path).unwrap();
                redirs.stderr = ErrorTarget::File(file);
            }
            _ => args.push(part.to_string()),
        }
    }

    (args, redirs)
}

fn shell_exec(args: &[String], redirs: &mut Redirections) -> io::Result<()> {
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

fn shell_launch(args: &[String], redirs: &mut Redirections) -> io::Result<()> {
    if args.is_empty() {
        eprintln!("shell_launch: args is empty");
        exit(exitcode::UNAVAILABLE)
    }
    let mut exec = Command::new(&args[0]);

    let mut child = exec
        .args(&args[1..])
        .stdin(redirs.stdin.to_stdio())
        .stdout(redirs.stdout.to_stdio())
        .stderr(redirs.stderr.to_stdio())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        if let InputSource::Pipe(reader) = &redirs.stdin {
            let reader = BufReader::new(reader);
            for line in reader.lines() {
                writeln!(stdin, "{}", line?)?;
            }
        }
    }

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
