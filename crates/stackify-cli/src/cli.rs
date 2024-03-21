use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{command, Args, ColorChoice, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;

use self::build::BuildArgs;
use self::clean::CleanArgs;
use self::config::ConfigArgs;
use self::env::args::EnvArgs;
use self::info::InfoArgs;

// Top-level command handlers
pub mod build;
pub mod clean;
pub mod config;
pub mod env;
pub mod network;
pub mod show;
pub mod info;
pub mod util;

pub mod clap_verbosity_flag;
pub mod clap_color_flag;

/// Command
#[derive(Debug, Parser)]
#[command(
    author = "Cyle Witruk (https://github.com/cylewitruk)", 
    version,
    about,
    long_about = None,
    styles=styles(),
    color = ColorChoice::Always
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbosity: Verbosity,

    #[command(flatten)]
    pub color: clap_color_flag::Color,
}

impl Cli {
    pub fn _validate(self) -> Result<Self> {
        Ok(self)
    }
}

/// Enum which defines our root commands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Builds all of the necessary artifacts used by the stackify tool. This
    /// includes Docker images, Bitcoin Core binaries, Stacks binaries, etc.
    Build(BuildArgs),
    /// Commands for configuring, manipulating and interacting with environments.
    #[clap(visible_alias = "env")]
    Environment(EnvArgs),
    /// Displays the current environment status.
    Info(InfoArgs),
    /// Cleans up resources created/used by stackify.
    Clean(CleanArgs),
    /// Commands for interacting with the stackify global configuration.
    Config(ConfigArgs),
}

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Red.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}
