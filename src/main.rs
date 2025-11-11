#[allow(unused_imports)]
use std::collections::HashMap;
use std::{
    env::{current_dir, home_dir, set_current_dir, split_paths, var_os},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use is_executable::IsExecutable;

struct Builtin {
    name: &'static str,
    func: fn(&[String]),
}

static BUILTINS: [Builtin; 5] = [
    Builtin {
        name: "exit",
        func: exit,
    },
    Builtin {
        name: "echo",
        func: echo,
    },
    Builtin {
        name: "type",
        func: exec_type,
    },
    Builtin {
        name: "pwd",
        func: pwd,
    },
    Builtin {
        name: "cd",
        func: cd,
    },
];

fn main() {
    loop {
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        for line in input.lines() {
            if let Ok(words) = shellwords::split(line) {
                if let Some(cmd) = words.first() {
                    if let Some(cmd) = BUILTINS.iter().find(|b| b.name == cmd) {
                        (cmd.func)(&words[1..]);
                    } else if let Some(cmd) = find_path_exec(cmd) {
                        if let Some(cmd) = cmd.file_name() {
                            let mut cmd = Command::new(cmd);
                            cmd.args(&words[1..]);
                            cmd.status().expect("error running the executable");
                        }
                    } else {
                        println!("{}: command not found", cmd);
                    }
                }
            }
        }
    }
}

fn find_path_exec<P>(exec: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    var_os("PATH").and_then(|paths| {
        split_paths(&paths).find_map(|dir| {
            let full_path = dir.join(&exec);
            if full_path.is_executable() {
                Some(full_path)
            } else {
                None
            }
        })
    })
}

fn exit(args: &[String]) {
    let code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or_default();
    std::process::exit(code);
}

fn echo(args: &[String]) {
    println!("{}", args.join(" "))
}

fn exec_type(args: &[String]) {
    if let Some(cmd) = args.first() {
        if let Some(builtin) = BUILTINS.iter().find(|b| b.name == cmd) {
            println!("{} is a shell builtin", builtin.name);
            return;
        }
        if let Some(exec) = find_path_exec(cmd) {
            println!("{} is {:#}", cmd, exec.display());
            return;
        }
        println!("{}: not found", cmd)
    }
}

fn pwd(_: &[String]) {
    println!("{}", current_dir().unwrap_or_default().display());
}

fn cd(args: &[String]) {
    fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
        let p = path.as_ref();

        if p.starts_with("~") {
            if let Some(home) = home_dir() {
                return home.join(p.strip_prefix("~").unwrap());
            }
        };
        p.to_path_buf()
    }

    if let Some(path) = args.first() {
        let path = expand_tilde(Path::new(path));
        if path.is_dir() {
            set_current_dir(path).unwrap();
        } else {
            println!("cd: {}: No such file or directory", path.display());
        }
    }
}
