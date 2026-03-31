use crate::shell::Shell;
use crate::shell::parse::Cmd;
use std::{env, path::PathBuf};

pub fn cd(shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.len() > 1 {
        eprintln!("cd: too many arguments");
        return;
    }

    let target = cmd.args.first().map(String::as_str).unwrap_or("~");

    let target_path = if target == "-" {
        shell.prev_cwd.clone()
    } else if target.starts_with('~') {
        let Some(home) = env::home_dir() else {
            eprintln!("cd: HOME not set");
            return;
        };

        if target == "~" {
            home
        } else if let Some(rest) = target.strip_prefix("~/") {
            home.join(rest)
        } else {
            eprintln!("cd: no such file or directory: {}", target);
            return;
        }
    } else {
        PathBuf::from(target)
    };

    let final_path = if target_path.is_absolute() {
        target_path
    } else {
        shell.cwd.join(target_path)
    };

    let old = shell.cwd.clone();
    match env::set_current_dir(&final_path) {
        Ok(_) => {
            shell.prev_cwd = old;
            shell.cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            if target == "-" {
                println!("{}", shell.cwd.to_string_lossy());
            }
            shell.update_prompt();
        }
        Err(_) => {
            eprintln!("cd: no such file or directory: {}", target);
        }
    }
}
