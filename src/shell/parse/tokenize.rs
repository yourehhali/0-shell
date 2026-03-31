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
        let raw_tokens = tokenize(input);
        let mut flags = Vec::new();
        let mut args = Vec::new();
        let flag_config = flag_config(&exec);
        let mut stop_parsing_flags = false;

        for token in raw_tokens {
            if token.is_empty() {
                continue;
            }

            if let Some(config) = flag_config {
                if !stop_parsing_flags && token == "--" {
                    stop_parsing_flags = true;
                    continue;
                }

                if config.allow_help && !stop_parsing_flags && token == "--help" {
                    flags.push(token);
                    continue;
                }

                if !stop_parsing_flags && token.starts_with('-') && token.len() > 1 {
                    let group = token.trim_start_matches('-');
                    if group.chars().all(|ch| config.allowed.contains(&ch)) {
                        flags.extend(group.chars().map(|ch| ch.to_string()));
                        continue;
                    }

                    return Err(format!("{}: invalid option -- '{}'\n", exec, group));
                }
            }

            args.push(token);
        }

        Ok((State::Exec, Cmd { exec, flags, args }))
    }
}

#[derive(Clone, Copy)]
struct FlagConfig {
    allow_help: bool,
    allowed: &'static [char],
}

fn flag_config(command_name: &str) -> Option<FlagConfig> {
    match command_name {
        "ls" => Some(FlagConfig {
            allow_help: true,
            allowed: &['l', 'a', 'F'],
        }),
        "rm" => Some(FlagConfig {
            allow_help: false,
            allowed: &['r', 'R'],
        }),
        _ => None,
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
                chars.next();
                if let Some(&escaped_char) = chars.peek() {
                    current.push(escaped_char);
                    chars.next();
                }
            }
            '\'' => {
                chars.next();
                if !in_double_quote {
                    in_single_quote = !in_single_quote;
                } else {
                    current.push(ch);
                }
            }
            '"' => {
                chars.next();
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
