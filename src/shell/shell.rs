use super::Shell;
use super::exec::*;
use super::parse::*;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::io::stdout;
use std::path::{Path, PathBuf};

use crossterm::cursor::MoveToColumn;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{event, execute};

use crate::shell::State;
use crate::shell::exec::helper::get_builtins;

impl Shell {
    pub fn new() -> Shell {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        Shell {
            cwd: cwd.clone(),
            prev_cwd: cwd,
            history: Vec::new(),

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
            format!(" \x1b[96mgit:(\x1b[92m{}\x1b[96m)", b)
        });

        self.prompt = format!(
            "➜ \x1b[1m \x1b[38;5;208m{}{} \x1b[38;5;208m$\x1b[0m ",
            display_name, branch_part
        );
    }

    pub fn run(mut self) {
        self.update_prompt();
        let parser = Parser::new();
        let executor = Executor::new();
        let mut input = String::new();

        print_banner();

        loop {
            if matches!(self.state, State::Exec) {
                self.state = State::Ready;
            }

            let prompt = match &self.state {
                State::Ready => self.prompt.as_str(),
                State::Quote(typ) => {
                    if typ == "dquote" {
                        "dquote> "
                    } else {
                        "quote> "
                    }
                }
                State::BackNewLine => "> ",
                State::Exec => self.prompt.as_str(),
            };

            let Some(line) = read_line_with_editing(prompt, &self.history) else {
                break;
            };

            if !input.is_empty() {
                input.push('\n');
            }
            input.push_str(&line);

            let state = parser.scan_command(&input);
            match state {
                State::Exec => match parser.parse_command(&input) {
                    Ok((state, cmd)) => match state {
                        State::Exec => {
                            self.state = State::Exec;
                            if !input.trim().is_empty() {
                                self.history.push(input.clone());
                            }
                            executor.execute_command(&mut self, &cmd);
                            input.clear();
                        }
                        _ => self.state = state,
                    },
                    Err(err) => print!("{err}"),
                },
                _ => self.state = state,
            };
        }
    }
}

fn read_line_with_editing(prompt: &str, history: &[String]) -> Option<String> {
    let mut stdout = stdout();
    if execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine)).is_err() {
        return None;
    }
    print!("{prompt}");
    let _ = stdout.flush();

    let _raw_mode = RawModeGuard::new()?;

    let mut buffer: Vec<char> = Vec::new();
    let mut cursor = 0usize;

    let mut history_index: Option<usize> = None;
    let mut history_draft: Vec<char> = Vec::new();

    loop {
        let ev = event::read().ok()?;
        let Event::Key(key) = ev else {
            continue;
        };

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('d') => {
                    if buffer.is_empty() {
                        let _ = execute!(stdout, MoveToColumn(0));
                        println!();
                        return None;
                    }
                }
                KeyCode::Char('c') => {
                    let _ = execute!(stdout, MoveToColumn(0));
                    println!();
                    return Some(String::new());
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Enter => {
                let _ = execute!(stdout, MoveToColumn(0));
                println!();
                return Some(buffer.iter().collect());
            }
            KeyCode::Char(ch) => {
                buffer.insert(cursor, ch);
                cursor += 1;
            }
            KeyCode::Backspace => {
                if cursor > 0 {
                    cursor -= 1;
                    buffer.remove(cursor);
                }
            }
            KeyCode::Delete => {
                if cursor < buffer.len() {
                    buffer.remove(cursor);
                }
            }
            KeyCode::Left => {
                cursor = cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                if cursor < buffer.len() {
                    cursor += 1;
                }
            }
            KeyCode::Home => cursor = 0,
            KeyCode::End => cursor = buffer.len(),
            KeyCode::Up => {
                if history.is_empty() {
                    continue;
                }

                if history_index.is_none() {
                    history_draft = buffer.clone();
                }

                let next = match history_index {
                    None => history.len().saturating_sub(1),
                    Some(i) => i.saturating_sub(1),
                };
                history_index = Some(next);
                buffer = history[next].chars().collect();
                cursor = buffer.len();
            }
            KeyCode::Down => {
                let Some(i) = history_index else {
                    continue;
                };

                if i + 1 >= history.len() {
                    history_index = None;
                    buffer = history_draft.clone();
                    cursor = buffer.len();
                } else {
                    let next = i + 1;
                    history_index = Some(next);
                    buffer = history[next].chars().collect();
                    cursor = buffer.len();
                }
            }
            _ => {}
        }

        let prompt_len = visible_len(prompt);
        let current_text: String = buffer.iter().collect();
        if execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine)).is_err() {
            return None;
        }
        print!("{prompt}{current_text}");
        let _ = execute!(stdout, MoveToColumn((prompt_len + cursor) as u16));
        let _ = stdout.flush();
    }
}

fn visible_len(s: &str) -> usize {
    let mut len = 0usize;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        len += 1;
    }
    len
}

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> Option<Self> {
        crossterm::terminal::enable_raw_mode().ok()?;
        Some(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
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
    while let Some(header) = iter.next() {
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
    let git_path = find_git_dir(cwd)?;

    let head_path = git_path.join("HEAD");

    let head_content = fs::read_to_string(head_path).ok()?;

    if head_content.starts_with("ref: ") {
        let ref_path = head_content.trim_start_matches("ref: ").trim();
        let branch_name = ref_path.rsplit('/').next()?;
        Some(branch_name.to_string())
    } else {
        Some(head_content.trim().chars().take(7).collect())
    }
}

fn find_git_dir(start: &Path) -> Option<PathBuf> {
    let mut current = start;

    loop {
        let candidate = current.join(".git");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if candidate.is_file() {
            if let Ok(contents) = fs::read_to_string(&candidate) {
                if let Some(stripped) = contents.strip_prefix("gitdir: ") {
                    let path_str = stripped.trim();
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
