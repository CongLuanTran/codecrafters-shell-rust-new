use os_pipe::{PipeReader, PipeWriter};
use std::fs::File;
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
    pub fn to_stdio(&self) -> Stdio {
        match self {
            InputSource::Stdin(_) => Stdio::inherit(),
            InputSource::File(_) => Stdio::piped(),
            InputSource::Pipe(_) => Stdio::piped(),
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
    pub fn to_stdio(&self) -> Stdio {
        match self {
            OutputTarget::Stdout(_) => Stdio::inherit(),
            OutputTarget::File(_) => Stdio::piped(),
            OutputTarget::Pipe(_) => Stdio::piped(),
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
    pub fn to_stdio(&self) -> Stdio {
        match self {
            ErrorTarget::Stderr(_) => Stdio::inherit(),
            ErrorTarget::File(_) => Stdio::piped(),
            ErrorTarget::Pipe(_) => Stdio::piped(),
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
