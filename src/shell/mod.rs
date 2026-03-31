use std::collections::HashMap;
use std::path::PathBuf;

pub mod exec;
pub mod parse;
#[allow(clippy::module_inception)]
pub mod shell;

pub type BuiltinFn = fn(&mut Shell, &parse::Cmd);

pub struct Shell {
    pub cwd: PathBuf,
    pub prev_cwd: PathBuf,
    pub history: Vec<String>,

    pub prompt: String,
    pub builtins: HashMap<String, BuiltinFn>,
    pub state: State,
}

pub enum State {
    Exec,
    Ready,
    Quote(String),
    BackNewLine,
}
