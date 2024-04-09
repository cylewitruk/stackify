use std::sync::Mutex;

use once_cell::sync::Lazy;

pub static LOG: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

macro_rules! clilog {
    ($($arg:tt)*) => {
        $crate::cli::log::LOG.lock().unwrap().push(format!($($arg)*))
    };
}

pub fn get_log() -> Vec<String> {
    LOG.lock().unwrap().clone()
}

pub(crate) use clilog;
