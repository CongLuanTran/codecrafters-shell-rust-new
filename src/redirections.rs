use os_pipe::{PipeReader, PipeWriter};
use std::fs::{File, OpenOptions};
use std::io::{stderr, stdin, stdout, Read, Result, Stderr, Stdin, Stdout, Write};
use std::process::Stdio;

pub enum InputSource {
    Stdin(Stdin),
    File(File),
    Pipe(PipeReader),
}

impl Read for InputSource {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            InputSource::Stdin(s) => s.read(buf),
            InputSource::File(f) => f.read(buf),
            InputSource::Pipe(p) => p.read(buf),
        }
    }
}

impl InputSource {
    pub fn to_stdio(&self) -> Result<Stdio> {
        match self {
            InputSource::Stdin(_) => Ok(Stdio::inherit()),
            InputSource::File(f) => Ok(Stdio::from(f.try_clone()?)),
            InputSource::Pipe(p) => Ok(Stdio::from(p.try_clone()?)),
        }
    }
}

pub enum OutputTarget {
    Stdout(Stdout),
    File(File),
    Pipe(PipeWriter),
}

impl Write for OutputTarget {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            OutputTarget::Stdout(s) => s.write(buf),
            OutputTarget::File(f) => f.write(buf),
            OutputTarget::Pipe(p) => p.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            OutputTarget::Stdout(s) => s.flush(),
            OutputTarget::File(f) => f.flush(),
            OutputTarget::Pipe(p) => p.flush(),
        }
    }
}

impl OutputTarget {
    pub fn to_stdio(&self) -> Result<Stdio> {
        match self {
            OutputTarget::Stdout(_) => Ok(Stdio::inherit()),
            OutputTarget::File(f) => Ok(Stdio::from(f.try_clone()?)),
            OutputTarget::Pipe(p) => Ok(Stdio::from(p.try_clone()?)),
        }
    }
}

pub enum ErrorTarget {
    Stderr(Stderr),
    File(File),
    Pipe(PipeWriter),
}

impl Write for ErrorTarget {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            ErrorTarget::Stderr(s) => s.write(buf),
            ErrorTarget::File(f) => f.write(buf),
            ErrorTarget::Pipe(p) => p.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            ErrorTarget::Stderr(s) => s.flush(),
            ErrorTarget::File(f) => f.flush(),
            ErrorTarget::Pipe(p) => p.flush(),
        }
    }
}

impl ErrorTarget {
    pub fn to_stdio(&self) -> Result<Stdio> {
        match self {
            ErrorTarget::Stderr(_) => Ok(Stdio::inherit()),
            ErrorTarget::File(f) => Ok(Stdio::from(f.try_clone()?)),
            ErrorTarget::Pipe(p) => Ok(Stdio::from(p.try_clone()?)),
        }
    }
}

pub struct Redirections {
    pub stdin: InputSource,
    pub stdout: OutputTarget,
    pub stderr: ErrorTarget,
}

impl Default for Redirections {
    fn default() -> Self {
        Self::new()
    }
}

impl Redirections {
    pub fn new() -> Self {
        Self {
            stdin: InputSource::Stdin(stdin()),
            stdout: OutputTarget::Stdout(stdout()),
            stderr: ErrorTarget::Stderr(stderr()),
        }
    }
}

// Handle redirections
pub fn parse_redirections(words: &[String]) -> (Vec<String>, Redirections) {
    let mut words = words.iter();
    let mut args = Vec::new();
    let mut redirs = Redirections::new();
    let mut openner = OpenOptions::new();
    let options = openner.create(true).write(true);
    while let Some(part) = words.next() {
        match part.as_str() {
            ">" | "1>" => {
                let path = words.next().unwrap();
                let file = options.write(true).truncate(true).open(path).unwrap();
                redirs.stdout = OutputTarget::File(file);
            }
            ">>" | "1>>" => {
                let path = words.next().unwrap();
                let file = options.truncate(false).append(true).open(path).unwrap();
                redirs.stdout = OutputTarget::File(file);
            }
            "2>" => {
                let path = words.next().unwrap();
                let file = options.write(true).truncate(true).open(path).unwrap();
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
