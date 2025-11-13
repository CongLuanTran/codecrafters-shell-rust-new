use std::fs::File;
use std::io::{stderr, stdout, Result, Stderr, Stdout, Write};
use std::process::Stdio;

pub enum OutputTarget {
    Stdout(Stdout),
    File(File),
}

impl Write for OutputTarget {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            OutputTarget::Stdout(s) => s.write(buf),
            OutputTarget::File(f) => f.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            OutputTarget::Stdout(s) => s.flush(),
            OutputTarget::File(f) => f.flush(),
        }
    }
}

impl OutputTarget {
    pub fn to_stdio(&self) -> Stdio {
        match self {
            OutputTarget::Stdout(_) => Stdio::inherit(),
            OutputTarget::File(_) => Stdio::piped(),
        }
    }
}

pub enum ErrorTarget {
    Stderr(Stderr),
    File(File),
}

impl Write for ErrorTarget {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            ErrorTarget::Stderr(s) => s.write(buf),
            ErrorTarget::File(f) => f.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            ErrorTarget::Stderr(s) => s.flush(),
            ErrorTarget::File(f) => f.flush(),
        }
    }
}

impl ErrorTarget {
    pub fn to_stdio(&self) -> Stdio {
        match self {
            ErrorTarget::Stderr(_) => Stdio::inherit(),
            ErrorTarget::File(_) => Stdio::piped(),
        }
    }
}

pub struct Redirections {
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
            stdout: OutputTarget::Stdout(stdout()),
            stderr: ErrorTarget::Stderr(stderr()),
        }
    }
}
