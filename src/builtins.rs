use crate::redirections::Redirections;
use is_executable::is_executable;
use std::env::split_paths;
use std::env::{current_dir, home_dir, set_current_dir, var_os};
use std::fs;
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Debug, Clone, Copy)]
pub struct Builtin {
    pub name: &'static str,
    pub func: fn(&[String], redirs: &mut Redirections) -> Result<()>,
}

pub static BUILTINS: [Builtin; 5] = [
    Builtin {
        name: "exit",
        func: shell_exit,
    },
    Builtin {
        name: "echo",
        func: shell_echo,
    },
    Builtin {
        name: "type",
        func: shell_type,
    },
    Builtin {
        name: "pwd",
        func: shell_pwd,
    },
    Builtin {
        name: "cd",
        func: shell_cd,
    },
];

fn shell_exit(args: &[String], _: &mut Redirections) -> Result<()> {
    let code = args.get(1).and_then(|s| s.parse().ok()).unwrap_or_default();
    exit(code);
}

pub fn shell_echo(args: &[String], redirs: &mut Redirections) -> Result<()> {
    writeln!(redirs.stdout, "{}", args[1..].join(" "))
}

pub fn list_path() -> Result<Vec<PathBuf>> {
    let mut exec = Vec::new();
    if let Some(paths) = var_os("PATH") {
        for path in split_paths(&paths) {
            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let path = entry?.path();
                    if path.is_file() && is_executable(&path) {
                        exec.push(path);
                    }
                }
            }
        }
    }

    Ok(exec)
}

fn shell_type(args: &[String], redirs: &mut Redirections) -> Result<()> {
    for cmd in args[1..].iter() {
        if BUILTINS.iter().any(|b| b.name == cmd) {
            writeln!(redirs.stdout, "{} is a shell builtin", cmd)?;
        } else if let Some(exec) = list_path()?
            .iter()
            .find(|p| p.file_name().unwrap().to_str().unwrap() == cmd)
        {
            writeln!(redirs.stdout, "{} is {}", cmd, exec.display())?;
        } else {
            writeln!(redirs.stdout, "{}: not found", cmd)?;
        }
    }
    Ok(())
}

fn shell_pwd(_: &[String], redirs: &mut Redirections) -> Result<()> {
    writeln!(
        redirs.stdout,
        "{}",
        current_dir().unwrap_or_default().display()
    )
}

fn shell_cd(args: &[String], redirs: &mut Redirections) -> Result<()> {
    fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
        let p = path.as_ref();

        if p.starts_with("~") {
            if let Some(home) = home_dir() {
                return home.join(p.strip_prefix("~").unwrap());
            }
        };
        p.to_path_buf()
    }

    let path = expand_tilde(Path::new(&args[1]));
    if path.is_dir() {
        set_current_dir(path)
    } else {
        writeln!(
            redirs.stdout,
            "cd: {}: No such file or directory",
            path.display()
        )
    }
}
