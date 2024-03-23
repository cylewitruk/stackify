use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

pub fn new_progressbar(template: &str, message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template(template)
            .unwrap()
            .tick_chars("/|\\- "),
    );
    pb.set_message(message.to_string());
    pb
}