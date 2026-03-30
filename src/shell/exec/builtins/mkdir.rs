use crate::shell::Shell;
use crate::shell::parse::Cmd;
use std::fs;

pub fn mkdir(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.is_empty() {
        eprintln!("mkdir: missing operand");
        return;
    }

    for p in &cmd.args {
        if let Err(e) = fs::create_dir(p) {
            eprintln!("mkdir: cannot create directory '{}': {}", p, e);
        }
    }
}
