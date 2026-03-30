pub mod builtins;
pub mod helper;

use super::{Shell, State};
use crate::shell::parse::Cmd;

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_command(&self, shell: &mut Shell, command: &Cmd) {
        match shell.builtins.get(&command.exec) {
            Some(func) => func(shell, command),
            None => println!("Command '{}' not found", command.exec.trim()),
        };

        shell.state = State::Ready;
    }
}
