use std::collections::HashMap;
use std::path::PathBuf;

pub mod exec;
pub mod parse;
pub mod shell;

pub type BuiltinFn = fn(&mut Shell, &parse::Cmd);

#[allow(dead_code)] //to remove it later when all field is used
pub struct Shell {
    pub pid: u32,                     // Shell's process ID
    pub cwd: PathBuf,                 // Current working directory
    pub env: HashMap<String, String>, // Environment variables
    pub history: Vec<String>,         // Command history
    pub last_status: i32,             // Exit status of the last command

    pub prompt: String,
    pub builtins: HashMap<String, BuiltinFn>, // store all built in cmd
    pub state: State,
}

pub enum State {
    Exec,
    Ready,
    Quote(String),
    BackNewLine,
}
