use crate::shell::State;

use super::*;

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_command(&self, input: &str) -> State {
        if input.ends_with("\\") && !input.ends_with("\\\\") {
            return State::BackNewLine;
        }

        let mut in_quote = None;
        let mut escaped = false;

        for c in input.chars() {
            if escaped {
                escaped = false;
                continue;
            }

            match c {
                '\\' => escaped = true,
                '"' | '\'' => match in_quote {
                    Some(q) if q == c => in_quote = None,
                    None => in_quote = Some(c),
                    _ => {}
                },
                _ => {}
            }
        }

        if let Some(q) = in_quote {
            if q == '\"' {
                State::Quote("dquote".to_string())
            } else {
                State::Quote("quote".to_string())
            }
        } else {
            State::Exec
        }
    }

    pub fn parse_command(&self, input: &str) -> Result<(State, Cmd), String> {
        let exec = match input.split_whitespace().nth(0) {
            Some(exe) => exe.to_string(),
            None => return Err("".to_owned()),
        };

        let input = input.trim_start_matches(&exec).trim();
        let raw_tokens = tokenize(&input);
        let mut flags = Vec::new();
        let mut args = Vec::new();

        for token in raw_tokens {
            if token == "--help" {
                flags.push(token);
                continue;
            }

            if token.starts_with("--") {
                flags.push(token);
                continue;
            }

            if token.starts_with('-') && token.len() > 1 {
                let new_vec: Vec<String> = token
                    .trim_start_matches('-')
                    .chars()
                    .map(|c| c.to_string())
                    .collect();

                if valid_flags(&new_vec) {
                    flags.extend(new_vec);
                } else {
                    return Err(format!(
                        "{}: invalid option -- '{}'\n",
                        exec,
                        token.trim_start_matches('-')
                    ));
                }
            } else if !token.is_empty() {
                args.push(token);
            }
        }

        Ok((
            State::Exec,
            Cmd {
                exec,
                flags,
                args,
            },
        ))
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(&ch) = chars.peek() {
        match ch {
            '\\' => {
                chars.next(); // consume '\'
                if let Some(&escaped_char) = chars.peek() {
                    current.push(escaped_char);
                    chars.next(); // consume escaped char
                }
            }
            '\'' => {
                chars.next(); // consume quote
                if !in_double_quote {
                    in_single_quote = !in_single_quote;
                } else {
                    current.push(ch);
                }
            }
            '"' => {
                chars.next(); // consume quote
                if !in_single_quote {
                    in_double_quote = !in_double_quote;
                } else {
                    current.push(ch);
                }
            }
            ' ' | '\t' => {
                if in_single_quote || in_double_quote {
                    current.push(ch);
                    chars.next();
                } else {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    while let Some(&space) = chars.peek() {
                        if space == ' ' || space == '\t' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
            }
            _ => {
                current.push(ch);
                chars.next();
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn valid_flags(_args: &Vec<String>) -> bool {
    true
}
