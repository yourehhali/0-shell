use crate::shell::Shell;
use crate::shell::parse::Cmd;
use std::collections::HashMap;
use std::io::{stdout, Write};

pub fn exit(_shell: &mut Shell, cmd: &Cmd) {
    print_exit_banner();
    if cmd.args.is_empty() {
        std::process::exit(0)
    };
    match cmd.args[0].parse::<i32>() {
        Ok(nb) => std::process::exit(nb),
        Err(_) => std::process::exit(0),
    };
}

pub fn echo(_shell: &mut Shell, cmd: &Cmd) {
    println!("{}", cmd.args.join(" "));
}

pub fn pwd(shell: &mut Shell, _cmd: &Cmd) {
    println!("{}", shell.cwd.to_str().unwrap_or(""));
}

pub fn clear(_shell: &mut Shell, _cmd: &Cmd) {
    print!("\x1B[2J\x1B[H");
    let _ = stdout().flush();
}

fn print_exit_banner() {
    let orange = "\x1b[38;5;208m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";

    let font = load_banner_font();
    let lines = render_banner_text("TILL NEXT TIME", &font);
    for line in lines {
        println!("{bold}{orange}{}{reset}", line);
    }
}

fn load_banner_font() -> HashMap<char, Vec<String>> {
    let data = include_str!("../../../banner.txt");
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
