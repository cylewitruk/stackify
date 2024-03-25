#![allow(dead_code)]

use color_eyre::owo_colors::OwoColorize;

pub fn print_ok(extra: Option<&str>) {
    if let Some(extra) = extra {
        println!("[{}] {}", "OK".green(), extra);
    } else {
        println!("[{}]", "OK".green());
    }
}

pub fn print_skip(extra: Option<&str>) {
    if let Some(extra) = extra {
        println!("[{}] {}", "SKIP".yellow(), extra);
    } else {
        println!("[{}]", "SKIP".yellow());
    }
}

pub fn print_fail(extra: Option<&str>) {
    if let Some(extra) = extra {
        println!("[{}] {}", "FAIL".red(), extra);
    } else {
        println!("[{}]", "FAIL".red());
    }
}

pub fn print_bytes(bytes: u64) -> String {
    let kb = bytes as f64 / 1024_f64;
    if kb < 1000_f64 {
        return format!("{:.2}KB", kb);
    }

    let mb = kb as f64 / 1024_f64;
    if mb < 1000_f64 {
        return format!("{:.2}MB", mb);
    }

    let gb = mb as f64 / 1024_f64;
    return format!("{:.2}GB", gb);
}
