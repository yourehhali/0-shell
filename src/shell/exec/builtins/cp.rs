use crate::shell::Shell;
use crate::shell::parse::Cmd;
use std::fs;

pub fn cp(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.len() < 2 {
        eprintln!("cp: missing file operand");
        return;
    }

    let src = &cmd.args[0];
    let dst = &cmd.args[1];

    match fs::metadata(src) {
        Ok(m) if m.is_dir() => {
            eprintln!("cp: -r not specified; omitting directory '{}'", src);
            return;
        }
        Ok(_) => {}
        Err(e) => {
            eprintln!("cp: cannot stat '{}': {}", src, e);
            return;
        }
    }

    if let Err(e) = fs::copy(src, dst) {
        eprintln!("cp: cannot copy '{}' to '{}': {}", src, dst, e);
    }
}
