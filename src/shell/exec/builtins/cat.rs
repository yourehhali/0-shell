use crate::shell::Shell;
use crate::shell::parse::Cmd;
use std::{fs, io};

pub fn cat(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.is_empty() {
        let _ = copy_stdin_to_stdout();
        return;
    }

    for file in &cmd.args {
        if file == "-" {
            let _ = copy_stdin_to_stdout();
            continue;
        }

        match fs::File::open(file) {
            Ok(mut f) => {
                let mut out = io::stdout().lock();
                let _ = io::copy(&mut f, &mut out);
            }
            Err(_) => eprintln!("cat: {}: No such file or directory", file),
        }
    }
}

fn copy_stdin_to_stdout() -> io::Result<u64> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    io::copy(&mut stdin, &mut stdout)
}
