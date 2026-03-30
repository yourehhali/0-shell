use crate::shell::Shell;
use crate::shell::parse::Cmd;
use std::fs;
use std::path::Path;

pub fn rm(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.is_empty() {
        eprintln!("rm: missing operand");
        return;
    }

    let recursive = cmd.flags.iter().any(|f| f == "r" || f == "R");

    for p in &cmd.args {
        let path = Path::new(p);
        match fs::symlink_metadata(path) {
            Ok(meta) => {
                if meta.is_dir() {
                    if recursive {
                        if let Err(e) = remove_dir_recursive(path) {
                            eprintln!("rm: cannot remove '{}': {}", p, e);
                        }
                    } else {
                        eprintln!("rm: cannot remove '{}': Is a directory", p);
                    }
                } else if let Err(e) = fs::remove_file(path) {
                    eprintln!("rm: cannot remove '{}': {}", p, e);
                }
            }
            Err(e) => eprintln!("rm: cannot remove '{}': {}", p, e),
        }
    }
}

fn remove_dir_recursive(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        let meta = fs::symlink_metadata(&p)?;
        if meta.is_dir() {
            remove_dir_recursive(&p)?;
        } else {
            fs::remove_file(&p)?;
        }
    }
    fs::remove_dir(dir)
}
