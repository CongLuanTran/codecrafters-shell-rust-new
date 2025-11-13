use crate::redirections::Redirections;
use is_executable::is_executable;
use std::env::split_paths;
use std::env::{current_dir, home_dir, set_current_dir, var_os};
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

fn shell_type(args: &[String], redirs: &mut Redirections) -> Result<()> {
    fn find_path<P>(cmd: P) -> Option<PathBuf>
    where
        P: AsRef<Path>,
    {
        var_os("PATH").and_then(|paths| {
            split_paths(&paths).find_map(|dir| {
                let path = dir.join(&cmd);
                if path.is_file() && is_executable(&path) {
                    Some(path)
                } else {
                    None
                }
            })
        })
    }
    for cmd in args[1..].iter() {
        if let Some(builtin) = BUILTINS.iter().find(|b| b.name == cmd) {
            writeln!(redirs.stdout, "{} is a shell builtin", builtin.name)?;
        } else if let Some(exec) = find_path(cmd) {
            writeln!(redirs.stdout, "{} is {:#}", cmd, exec.display())?;
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
