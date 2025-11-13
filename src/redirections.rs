use std::io::{Read, Write};

#[derive(Default)]
pub struct Redirections {
    pub stdin: Option<Box<dyn Read>>,
    pub stdout: Option<Box<dyn Write>>,
    pub stderr: Option<Box<dyn Write>>,
}

impl Redirections {
    pub fn new() -> Self {
        Self::default()
    }
}
