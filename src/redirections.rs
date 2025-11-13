use std::fs::{File, OpenOptions};
use std::io::{stderr, stdout, Error, Result, Stderr, Stdout, Write};
use std::path::PathBuf;
use std::process::Stdio;

pub enum OutputTarget {
    Stdout(Stdout),
    File {
        path: PathBuf,
        handle: Option<File>,
        append: bool,
    },
}

impl Write for OutputTarget {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            OutputTarget::Stdout(s) => s.write(buf),
            OutputTarget::File { .. } => self.ensure_file()?.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            OutputTarget::Stdout(s) => s.flush(),
            OutputTarget::File { .. } => self.ensure_file()?.flush(),
        }
    }
}

impl OutputTarget {
    pub fn to_stdio(&self) -> Stdio {
        match self {
            OutputTarget::Stdout(_) => Stdio::inherit(),
            OutputTarget::File { path, .. } => Stdio::from(File::create(path).unwrap()),
        }
    }
    fn ensure_file(&mut self) -> Result<&mut std::fs::File> {
        match self {
            OutputTarget::Stdout(_) => Err(Error::other("stdout")),
            OutputTarget::File {
                path,
                handle,
                append,
            } => {
                if handle.is_none() {
                    *handle = Some(
                        OpenOptions::new()
                            .write(true)
                            .create(true)
                            .append(*append)
                            .truncate(!*append)
                            .open(path)?,
                    );
                }
                Ok(handle.as_mut().unwrap())
            }
        }
    }
}

pub enum ErrorTarget {
    Stderr(Stderr),
    File {
        path: PathBuf,
        handle: Option<File>,
        append: bool,
    },
}

impl Write for ErrorTarget {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            ErrorTarget::Stderr(s) => s.write(buf),
            ErrorTarget::File { .. } => self.ensure_file()?.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            ErrorTarget::Stderr(s) => s.flush(),
            ErrorTarget::File { .. } => self.ensure_file()?.flush(),
        }
    }
}

impl ErrorTarget {
    pub fn to_stdio(&self) -> Stdio {
        match self {
            ErrorTarget::Stderr(_) => Stdio::inherit(),
            ErrorTarget::File { path, .. } => Stdio::from(File::create(path).unwrap()),
        }
    }
    fn ensure_file(&mut self) -> Result<&mut std::fs::File> {
        match self {
            ErrorTarget::Stderr(_) => Err(Error::other("stderr")),
            ErrorTarget::File {
                path,
                handle,
                append,
            } => {
                if handle.is_none() {
                    *handle = Some(
                        OpenOptions::new()
                            .write(true)
                            .create(true)
                            .append(*append)
                            .truncate(!*append)
                            .open(path)?,
                    );
                }
                Ok(handle.as_mut().unwrap())
            }
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
