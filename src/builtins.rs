use crate::readline::MyShell;
use crate::redirections::Redirections;
use is_executable::is_executable;
use rustyline::history::History;
use std::env::split_paths;
use std::env::{current_dir, home_dir, set_current_dir, var_os};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{fs, io};

#[derive(Debug, Clone, Copy)]
pub struct Builtin {
    pub name: &'static str,
    pub func: fn(shell: &mut MyShell, args: &[String], redirs: &mut Redirections) -> Result<()>,
}

pub static BUILTINS: [Builtin; 7] = [
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
    Builtin {
        name: "history",
        func: shell_history,
    },
    Builtin {
        name: "jobs",
        func: shell_jobs,
    },
];

fn shell_exit(shell: &mut MyShell, args: &[String], _: &mut Redirections) -> Result<()> {
    let code = args.get(1).and_then(|s| s.parse().ok()).unwrap_or_default();
    if let Some(histfile) = var_os("HISTFILE") {
        if !histfile.is_empty() {
            shell.append_history(&histfile).map_err(io::Error::other)?;
        }
    }
    exit(code);
}

pub fn shell_echo(_: &mut MyShell, args: &[String], redirs: &mut Redirections) -> Result<()> {
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

fn shell_type(_: &mut MyShell, args: &[String], redirs: &mut Redirections) -> Result<()> {
    for cmd in args[1..].iter() {
        if cmd == "history" || BUILTINS.iter().any(|b| b.name == cmd) {
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

fn shell_pwd(_: &mut MyShell, _: &[String], redirs: &mut Redirections) -> Result<()> {
    writeln!(
        redirs.stdout,
        "{}",
        current_dir().unwrap_or_default().display()
    )
}

fn shell_cd(_: &mut MyShell, args: &[String], redirs: &mut Redirections) -> Result<()> {
    fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
        let p = path.as_ref();

        if p.starts_with("~") {
            if let Some(home) = home_dir() {
                return home.join(p.strip_prefix("~").unwrap());
            }
        };
        p.to_path_buf()
    }

    if args.len() == 1 {
        if let Some(home) = home_dir() {
            return set_current_dir(home);
        }
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

fn shell_history(shell: &mut MyShell, args: &[String], redirs: &mut Redirections) -> Result<()> {
    if args.len() == 1 {
        for (i, entry) in shell.history().into_iter().enumerate() {
            writeln!(redirs.stdout, "\t{} {}", i + 1, entry)?
        }
    } else if args.len() == 2 {
        if let Ok(num) = args[1].parse::<usize>() {
            let skip = shell.history().len().saturating_sub(num);
            for (i, entry) in shell.history().into_iter().enumerate().skip(skip) {
                writeln!(redirs.stdout, "\t{} {}", i + 1, entry)?
            }
        }
    } else if args.len() == 3 {
        match args[1].as_str() {
            "-r" => shell.load_history(&args[2]).map_err(io::Error::other)?,
            "-w" => shell.save_history(&args[2]).map_err(io::Error::other)?,
            "-a" => shell.append_history(&args[2]).map_err(io::Error::other)?,
            _ => {}
        };
    };

    Ok(())
}

fn shell_jobs(_: &mut MyShell, _: &[String], redirs: &mut Redirections) -> Result<()> {
    Ok(())
}
