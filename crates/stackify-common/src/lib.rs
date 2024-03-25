pub mod docker;

#[cfg(test)]
pub mod tests;
pub mod util;
pub mod download;
pub mod api;

use std::fmt::Display;

use color_eyre::eyre::{bail, Result};
use regex::Regex;


#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnvironmentName {
    // [a-z0-9]+(?:[._-]{1,2}[a-z0-9]+)*
    name: String,
}

impl EnvironmentName {
    pub fn new(name: &str) -> Result<Self> {
        // This is the Regex used by Docker for names.
        let regex = Regex::new("[a-z0-9]+(?:[._-]{1,2}[a-z0-9]+)*")?;
        if !regex.is_match(name) {
            bail!(format!("The environment name '{}' is invalid.", name));
        }
        
        Ok(Self {
            name: name.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl Display for EnvironmentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl AsRef<str> for EnvironmentName {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl Into<String> for EnvironmentName {
    fn into(self) -> String {
        self.name
    }
}

impl Into<String> for &EnvironmentName {
    fn into(self) -> String {
        self.name.clone()
    }
}

impl TryFrom<&str> for EnvironmentName {
    type Error = color_eyre::eyre::Error;

    fn try_from(name: &str) -> Result<Self> {
        Self::new(name)
    }
}

impl TryFrom<String> for EnvironmentName {
    type Error = color_eyre::eyre::Error;

    fn try_from(name: String) -> Result<Self> {
        Self::new(&name)
    }
}

