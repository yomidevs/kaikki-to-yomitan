use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::lang::{Edition, Lang};

pub const SKIP_C: &str = "⏭";
pub const CHECK_C: &str = "✓";

fn size(path: &Path) -> std::io::Result<u64> {
    let md = fs::metadata(path)?;
    if md.is_file() {
        Ok(md.len())
    } else if md.is_dir() {
        let mut total = 0;
        for entry in fs::read_dir(path)? {
            total += size(&entry?.path())?;
        }
        Ok(total)
    } else {
        // symlinks and other beasts
        Ok(0)
    }
}

pub fn human_size(size_bytes: f64) -> String {
    let mut size = size_bytes;
    for unit in ["B", "KB", "MB"] {
        if size < 1024.0 {
            return format!("{size:.2} {unit}");
        }
        size /= 1024.0;
    }
    format!("{size:.2} GB")
}

fn get_file_size_human(path: &Path) -> Result<String> {
    Ok(human_size(size(path)? as f64))
}

fn pretty_msg_at_path(msg: &str, path: &Path) -> String {
    let at = "\x1b[1;36m@\x1b[0m"; // bold + cyan
    match get_file_size_human(path) {
        Result::Ok(size_mb) => {
            let size_str = format!("\x1b[1m{size_mb}\x1b[0m"); // bold
            format!("{msg} {at} {} ({})", path.display(), size_str)
        }
        // Happens when we write to zip
        Err(..) => format!("{msg} {at} {}", path.display()),
    }
}

pub fn pretty_println_at_path(msg: &str, path: &Path) {
    println!("{}", pretty_msg_at_path(msg, path));
}

pub fn pretty_print_at_path(msg: &str, path: &Path) {
    print!("{}", pretty_msg_at_path(msg, path));
}

pub fn skip_because_file_exists(skipped: &str, path: &Path) {
    let msg = format!("{SKIP_C} Skipping {skipped}: file already exists");
    pretty_println_at_path(&msg, path);
}

/// Return a link to the wiktionary page of this word.
pub fn link_wiktionary(edition: Edition, source: Lang, word: &str) -> String {
    format!(
        "https://{}.wiktionary.org/wiki/{}#{}",
        edition,
        word.replace(' ', "%20"),
        source.long()
    )
}

/// Return a link to the kaikki page of this word.
pub fn link_kaikki(edition: Edition, source: Lang, word: &str) -> String {
    // 楽しい >> 楽/楽し/楽しい
    // 伸す >> 伸/伸す/伸す (when word.chars().count() < 2)
    // up >> u/up/up (word.len() is irrelevant, only char count matters)
    let chars: Vec<_> = word.chars().collect();
    let first = chars[0]; // word can't be empty
    let first_two = if chars.len() < 2 {
        word.to_string()
    } else {
        chars[0..2].iter().collect::<String>()
    };

    let dictionary = match edition {
        Edition::En => "dictionary",
        other => &format!("{other}wiktionary"),
    };
    let localized_source = match edition {
        Edition::En | Edition::El => &source.long().replace(' ', "%20"),
        // https://github.com/tatuylonen/wiktextract/issues/1497
        _ => "All%20languages%20combined",
    };

    format!(
        "https://kaikki.org/{dictionary}/{localized_source}/meaning/{first}/{first_two}/{word}.html"
    )
}
