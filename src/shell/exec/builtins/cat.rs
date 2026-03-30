use crate::shell::Shell;
use crate::shell::parse::Cmd;
use std::{fs, io};

pub fn cat(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.is_empty() {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            stdin.read_line(&mut input).unwrap();
            print!("{}", input);
        }
    } else {
        for file in &cmd.args {
            let content = fs::read_to_string(file);
            match content {
                Ok(data) => print!("{}", data),
                Err(_) => println!("cat: {}: No such file or directory", file),
            };
        }
    }
}
