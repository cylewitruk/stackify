#![allow(dead_code)]
// Taken from https://crates.io/crates/clap-color-flag

//! Control color with a `--color` flag for your CLI
//!
//! # Examples
//!
//! To get `--color` through your entire program, just `flatten` [`Color`]:
//! ```rust,no_run
//! use clap::Parser;
//! use clap_color_flag::Color;
//!
//! /// Le CLI
//! #[derive(Debug, Parser)]
//! struct Cli {
//!     #[command(flatten)]
//!     color: Color,
//! }
//! ```
//!
//! You can then use this to configure your formatter:
//! ```rust,no_run
//! use anstream::println;
//! use owo_colors::OwoColorize as _;
//! # use clap::Parser;
//! # use clap_color_flag::Color;
//! #
//! # /// Le CLI
//! # #[derive(Debug, Parser)]
//! # struct Cli {
//! #     #[command(flatten)]
//! #     color: Color,
//! # }
//!
//! let cli = Cli::parse();
//!
//! cli.color.write_global();
//!
//! println!("Hello, {}!", "world".red());
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use clap::ColorChoice;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, clap::Args)]
pub struct Color {
    /// Controls when to use color.
    #[arg(
        long,
        default_value_t = ColorChoice::Auto,
        value_name = "WHEN",
        value_enum,
        global = true,
        help_heading = "Other"
    )]
    color: ColorChoice,
}

impl Color {
    /// Set the user selection on `colorchoice`
    pub fn write_global(&self) {
        self.as_choice().write_global();
    }

    /// Get the user's selection
    pub fn as_choice(&self) -> colorchoice::ColorChoice {
        match self.color {
            ColorChoice::Auto => colorchoice::ColorChoice::Auto,
            ColorChoice::Always => colorchoice::ColorChoice::Always,
            ColorChoice::Never => colorchoice::ColorChoice::Never,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_app() {
        #[derive(Debug, clap::Parser)]
        struct Cli {
            #[command(flatten)]
            color: Color,
        }

        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
