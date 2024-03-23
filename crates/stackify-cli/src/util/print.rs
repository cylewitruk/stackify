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