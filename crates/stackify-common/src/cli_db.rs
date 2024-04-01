use color_eyre::Result;

use crate::types::Environment;

pub trait CliDatabase {
    fn load_environment(&self, name: &str) -> Result<Environment>;
}
