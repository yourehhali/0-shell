use chrono::{DateTime, Duration, Local};
use std::fs;
use std::fs::Metadata;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use users::{get_group_by_gid, get_user_by_uid};

use crate::shell::Shell;
use crate::shell::parse::Cmd;

#[derive(Default, Clone, Copy)]
struct LsOptions {
    show_hidden: bool,
    long_format: bool,
    f_type: bool,
}

pub fn ls(_shell: &mut Shell, cmd: &Cmd) {
    let mut options = LsOptions::default();

    for flag in &cmd.flags {
        match flag.as_str() {
            "l" | "-l" => options.long_format = true,
            "a" | "-a" => options.show_hidden = true,
            "F" | "-F" => options.f_type = true,
            "--help" => {
                eprintln!("Usage: ls [OPTION]... [FILE]...\nList information about the FILEs (the current directory by default).\n\nOptions:\n  -l      use a long listing format\n  -a      do not ignore entries starting with .\n  -F      append indicator (one of */=>@|) to entries\n  --help  display this help and exit");
                return;
            }
            _ => {}
        }
    }

    let mut paths: Vec<PathBuf> = if cmd.args.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cmd.args.iter().map(PathBuf::from).collect()
    };

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for p in &paths {
        let metadata = match fs::symlink_metadata(p) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ls: cannot access '{}': {}", p.display(), e);
                continue;
            }
        };

        if metadata.is_dir() {
            directories.push(p.clone());
        } else {
            files.push(p.clone());
        }
    }

    let more_paths = directories.len() > 1 || (!files.is_empty() && !directories.is_empty());

    for file in &files {
        list_single(file, options);
    }

    for (i, dir) in directories.iter().enumerate() {
        if i > 0 || !files.is_empty() {
            println!();
        }
        list_directory(dir, options, more_paths);
    }

    paths.clear();
}

fn list_single(path: &Path, options: LsOptions) {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("ls: cannot access '{}': {}", path.display(), e);
            return;
        }
    };

    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    if options.long_format {
        let widths = MaxWidths::from_entries(&[(name.clone(), meta.clone(), path.to_path_buf())]);
        if let Some(line) = format_entry(&name, &meta, path, options, &widths) {
            print!("{}", line);
        }
    } else if let Some(line) = format_entry(&name, &meta, path, options, &MaxWidths::default()) {
        println!("{}", line.trim_end());
    }
}

fn list_directory(path: &Path, options: LsOptions, more_paths: bool) {
    let dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("ls: cannot access '{}': {}", path.display(), e);
            return;
        }
    };

    if more_paths {
        println!("{}:", path.display());
    }

    let mut entries: Vec<(String, Metadata, PathBuf)> = Vec::new();
    let mut total_blocks = 0u64;

    if options.show_hidden {
        for special in [".", ".."] {
            let special_path = path.join(special);
            if let Ok(meta) = fs::symlink_metadata(&special_path) {
                total_blocks += meta.blocks();
                entries.push((special.to_string(), meta, special_path));
            }
        }
    }

    for item in dir.flatten() {
        let name = item.file_name().to_string_lossy().to_string();
        if !options.show_hidden && name.starts_with('.') {
            continue;
        }

        let meta = match fs::symlink_metadata(item.path()) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ls: error reading metadata for '{}': {}", name, e);
                continue;
            }
        };

        total_blocks += meta.blocks();
        entries.push((name, meta, item.path()));
    }

    entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    if options.long_format {
        println!("total {}", total_blocks / 2);
        let widths = MaxWidths::from_entries(&entries);
        for (name, meta, p) in entries {
            if let Some(line) = format_entry(&name, &meta, &p, options, &widths) {
                print!("{}", line);
            }
        }
    } else {
        let widths = MaxWidths::default();
        for (name, meta, p) in entries {
            if let Some(line) = format_entry(&name, &meta, &p, options, &widths) {
                println!("{}", line.trim_end());
            }
        }
    }
}

#[derive(Default)]
struct MaxWidths {
    links: usize,
    user: usize,
    group: usize,
    size: usize,
    time: usize,
    perm: usize,
}

impl MaxWidths {
    fn from_entries(items: &[(String, Metadata, PathBuf)]) -> Self {
        let mut w = MaxWidths::default();
        for (_name, meta, path) in items {
            w.links = w.links.max(meta.nlink().to_string().len());

            let user = get_user_by_uid(meta.uid())
                .map(|u| u.name().to_string_lossy().len())
                .unwrap_or_else(|| meta.uid().to_string().len());
            w.user = w.user.max(user);

            let group = get_group_by_gid(meta.gid())
                .map(|g| g.name().to_string_lossy().len())
                .unwrap_or_else(|| meta.gid().to_string().len());
            w.group = w.group.max(group);

            let size = size_string(meta).len();
            w.size = w.size.max(size);

            let time = time_string(meta).len();
            w.time = w.time.max(time);

            let perm = format_permissions(meta.permissions().mode(), path).len() + 1;
            w.perm = w.perm.max(perm);
        }
        w
    }
}

fn format_entry(
    name: &str,
    meta: &Metadata,
    path: &Path,
    options: LsOptions,
    widths: &MaxWidths,
) -> Option<String> {
    let file_type = meta.file_type();
    let file_type_char = if file_type.is_symlink() {
        'l'
    } else if file_type.is_dir() {
        'd'
    } else if file_type.is_fifo() {
        'p'
    } else if file_type.is_socket() {
        's'
    } else if file_type.is_block_device() {
        'b'
    } else if file_type.is_char_device() {
        'c'
    } else {
        '-'
    };

    let (color, indicator) = if file_type.is_symlink() {
        ("\x1b[1;36m", "@")
    } else if file_type.is_dir() {
        ("\x1b[1;34m", "/")
    } else if meta.permissions().mode() & 0o111 != 0 {
        ("\x1b[1;32m", "*")
    } else if file_type.is_fifo() {
        ("\x1b[1;33m", "|")
    } else if file_type.is_socket() {
        ("\x1b[1;35m", "=")
    } else {
        ("\x1b[0m", "")
    };
    let reset = "\x1b[0m";

    if options.long_format {
        let perms = format_permissions(meta.permissions().mode(), path);
        let links = meta.nlink();
        let user = get_user_by_uid(meta.uid())
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| meta.uid().to_string());
        let group = get_group_by_gid(meta.gid())
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| meta.gid().to_string());
        let size = size_string(meta);
        let time = time_string(meta);

        let mut filename_display = format!("{}{}{}", color, name, reset);
        if file_type.is_symlink() {
            if let Ok(target) = fs::read_link(path) {
                filename_display = format!("{}{}{} -> {}", color, name, reset, target.display());
            }
        }

        let maybe_indicator = if options.f_type && indicator == "@" {
            ""
        } else if options.f_type {
            indicator
        } else {
            ""
        };

        Some(format!(
            "{}{:<perm_w$} {:>links_w$} {:<user_w$} {:<group_w$} {:>size_w$} {:>time_w$} {}{}\n",
            file_type_char,
            perms,
            links,
            user,
            group,
            size,
            time,
            filename_display,
            maybe_indicator,
            perm_w = widths.perm.saturating_sub(1),
            links_w = widths.links,
            user_w = widths.user,
            group_w = widths.group,
            size_w = widths.size,
            time_w = widths.time,
        ))
    } else {
        if options.f_type {
            Some(format!("{}{}{}{} ", color, name, reset, indicator))
        } else {
            Some(format!("{}{}{} ", color, name, reset))
        }
    }
}

fn format_permissions(mode: u32, _path: &Path) -> String {
    let mut res = String::new();

    let bits = [
        (mode & 0o400 != 0, 'r'),
        (mode & 0o200 != 0, 'w'),
        (mode & 0o100 != 0, 'x'),
        (mode & 0o040 != 0, 'r'),
        (mode & 0o020 != 0, 'w'),
        (mode & 0o010 != 0, 'x'),
        (mode & 0o004 != 0, 'r'),
        (mode & 0o002 != 0, 'w'),
        (mode & 0o001 != 0, 'x'),
    ];

    for (set, ch) in bits {
        res.push(if set { ch } else { '-' });
    }

    if mode & 0o4000 != 0 {
        res.replace_range(
            2..3,
            if &res[2..3] == "x" { "s" } else { "S" },
        );
    }
    if mode & 0o2000 != 0 {
        res.replace_range(
            5..6,
            if &res[5..6] == "x" { "s" } else { "S" },
        );
    }
    if mode & 0o1000 != 0 {
        res.replace_range(
            8..9,
            if &res[8..9] == "x" { "t" } else { "T" },
        );
    }

    res
}

fn size_string(meta: &Metadata) -> String {
    if meta.file_type().is_char_device() || meta.file_type().is_block_device() {
        let rdev = meta.rdev();
        let major = ((rdev >> 8) & 0xfff) as u32;
        let minor = ((rdev & 0xff) | ((rdev >> 12) & 0xfff00)) as u32;
        format!("{}, {}", major, minor)
    } else {
        meta.len().to_string()
    }
}

fn time_string(meta: &Metadata) -> String {
    let Ok(modified) = meta.modified() else {
        return String::new();
    };
    let date_time: DateTime<Local> = modified.into();
    let now = Local::now();

    if now.signed_duration_since(date_time) > Duration::days(180) || date_time > now {
        date_time.format("%b %e  %Y").to_string()
    } else {
        date_time.format("%b %e %H:%M").to_string()
    }
}
