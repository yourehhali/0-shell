mod tokenize;

pub use tokenize::Parser;

pub struct Cmd {
    pub exec: String,
    #[allow(dead_code)]
    pub flags: Vec<String>,
    pub args: Vec<String>,
}
