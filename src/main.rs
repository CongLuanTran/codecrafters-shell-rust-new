#[allow(unused_imports)]
use std::collections::HashMap;
use std::{
    env::{split_paths, var_os},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use is_executable::IsExecutable;

struct Builtin {
    name: &'static str,
    func: fn(&str),
}

static BUILTINS: [Builtin; 3] = [
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
];

fn main() {
    loop {
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        for line in input.lines() {
            if let Some((cmd, args)) = line.split_once(" ") {
                if let Some(builtin) = BUILTINS.iter().find(|b| b.name == cmd) {
                    (builtin.func)(args);
                    continue;
                }
            }
            println!("{}: command not found", line);
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

fn exit(args: &str) {
    let code = args.parse::<i32>().unwrap_or_default();
    std::process::exit(code);
}

fn echo(args: &str) {
    println!("{}", args)
}
fn exec_type(args: &str) {
    if let Some(builtin) = BUILTINS.iter().find(|b| b.name == args) {
        println!("{} is a shell builtin", builtin.name);
        return;
    }
    if let Some(exec) = find_path_exec(args) {
        println!("{} is {:#}", args, exec.display());
        return;
    }
    println!("{}: not found", args)
}
