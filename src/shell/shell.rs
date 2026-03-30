use super::Shell;
use super::exec::*;
use super::parse::*;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::io::{stdin, stdout};
use std::path::{Path, PathBuf};

use crate::shell::State;
use crate::shell::exec::helper::get_builtins;

impl Shell {
    pub fn new() -> Shell {
        Shell {
            pid: std::process::id(),
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            env: env::vars().collect(),
            history: Vec::new(),
            last_status: 0,

            prompt: "$ ".to_string(),
            builtins: get_builtins(),
            state: State::Ready,
        }
    }

    pub fn update_prompt(&mut self) {
        let display_path = self.cwd.clone();
        let pwd_path = display_path.to_str().unwrap_or("");
        let home_dir = env::home_dir()
            .and_then(|p| p.to_str().map(|s| s.to_owned()))
            .unwrap_or_else(|| String::from(""));

        let last_segment = if pwd_path == home_dir {
            "~"
        } else {
            self.cwd
                .file_name()
                .and_then(|os_str| os_str.to_str())
                .unwrap_or("")
        };

        let display_name = if last_segment.is_empty() {
            pwd_path.replace(&home_dir, "~")
        } else {
            last_segment.to_string()
        };

        let git_branch = get_git_branch(&self.cwd);
        let branch_part = git_branch.map_or(String::new(), |b| {
            format!(" \x1b[31mgit:(\x1b[36m{}\x1b[31m)", b)
        });

        self.prompt = format!(
            "\x1b[1m \x1b[38;5;208m{}{} \x1b[38;5;208m➜\x1b[0m ",
            display_name, branch_part
        );
    }

    pub fn run(mut self) {
        self.update_prompt();
        let parser = Parser::new();
        let executor = Executor::new();
        let stdin = stdin();
        let mut input = String::new();

        print_banner();

        loop {
            match &self.state {
                State::Exec => continue,
                State::Ready => {
                    print!("{}", self.prompt);
                    stdout().flush().unwrap();
                    input = String::new();
                }
                State::Quote(typ) => {
                    print!("{}> ", typ);
                    self.state = State::Ready;
                    stdout().flush().unwrap();
                }
                State::BackNewLine => {
                    print!("> ");
                    self.state = State::Ready;
                    stdout().flush().unwrap();
                }
            };

            if input.len() > 0 {
                let mut temp_buffer = String::new();
                match stdin.read_line(&mut temp_buffer) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
                input = format!("{}{}", input, temp_buffer);
            } else {
                match stdin.read_line(&mut input) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }

            let input = input.trim();
            let state = parser.scan_command(&input.trim());
            match state {
                State::Exec => match parser.parse_command(&input) {
                    Ok((state, cmd)) => {
                        match state {
                            State::Exec => {
                                self.state = State::Exec;
                                executor.execute_command(&mut self, &cmd);
                            }
                            _ => self.state = state,
                        }
                    }
                    Err(err) => print!("{err}"),
                },
                _ => self.state = state,
            };
        }
    }
}

fn print_banner() {
    let orange = "\x1b[38;5;208m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";

    let font = load_banner_font();
    let lines = render_banner_text("0 - shell", &font);
    for line in lines {
        println!("{bold}{orange}{}{reset}", line);
    }
    println!();
}

fn load_banner_font() -> HashMap<char, Vec<String>> {
    let data = include_str!("../banner.txt");
    let mut font: HashMap<char, Vec<String>> = HashMap::new();

    let mut iter = data.lines();
    loop {
        let Some(header) = iter.next() else { break };
        if header.is_empty() {
            continue;
        }

        if header.chars().count() != 1 {
            continue;
        }
        let ch = header.chars().next().unwrap();

        let mut glyph = Vec::new();
        for _ in 0..5 {
            if let Some(line) = iter.next() {
                glyph.push(line.to_string());
            } else {
                break;
            }
        }
        if glyph.len() == 5 {
            font.insert(ch, glyph);
        }
    }

    font
}

fn render_banner_text(text: &str, font: &HashMap<char, Vec<String>>) -> Vec<String> {
    let height = 5usize;
    let mut output = vec![String::new(); height];

    for c in text.chars() {
        let key = if c.is_ascii_alphabetic() {
            c.to_ascii_uppercase()
        } else {
            c
        };

        let glyph = font
            .get(&key)
            .or_else(|| font.get(&' '))
            .cloned()
            .unwrap_or_else(|| vec!["".to_string(); height]);

        for (i, row) in glyph.into_iter().enumerate().take(height) {
            if !output[i].is_empty() {
                output[i].push(' ');
            }
            output[i].push_str(&row);
        }
    }

    output
}

fn get_git_branch(cwd: &Path) -> Option<String> {
    // Find the .git directory, which might be a folder or a file (in case of submodules or worktrees)
    let git_path = find_git_dir(cwd)?;

    let head_path = git_path.join("HEAD");

    let head_content = fs::read_to_string(head_path).ok()?;

    // Typical content:
    // ref: refs/heads/main
    if head_content.starts_with("ref: ") {
        let ref_path = head_content.trim_start_matches("ref: ").trim();
        let branch_name = ref_path.rsplit('/').next()?;
        Some(branch_name.to_string())
    } else {
        // Detached HEAD or commit hash, just return first 7 chars as a short SHA
        Some(head_content.trim().chars().take(7).collect())
    }
}

// Recursively look for .git folder
fn find_git_dir(start: &Path) -> Option<PathBuf> {
    let mut current = start;

    loop {
        let candidate = current.join(".git");
        if candidate.is_dir() {
            return Some(candidate);
        }
        // Sometimes .git is a file pointing to another git dir (git worktrees etc)
        if candidate.is_file() {
            // Read the gitdir path inside the file
            if let Ok(contents) = fs::read_to_string(&candidate) {
                // Usually something like: gitdir: /path/to/actual/git/dir
                if contents.starts_with("gitdir: ") {
                    let path_str = contents["gitdir: ".len()..].trim();
                    let gitdir_path = if Path::new(path_str).is_absolute() {
                        PathBuf::from(path_str)
                    } else {
                        current.join(path_str)
                    };
                    return Some(gitdir_path);
                }
            }
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    None
}
