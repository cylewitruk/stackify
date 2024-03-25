use std::fmt::Display;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{command, Parser, Subcommand};
use clap_complete::Shell;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
use console::{style, StyledObject};
use lazy_static::lazy_static;

use self::clean::CleanArgs;
use self::config::args::ConfigArgs;
use self::env::args::EnvArgs;
use self::info::InfoArgs;
use self::init::InitArgs;

// Top-level command handlers
pub mod clean;
pub mod config;
pub mod env;
pub mod info;
pub mod init;
pub mod network;
pub mod show;

pub mod clap_color_flag;
pub mod clap_verbosity_flag;

pub const PAD_WIDTH: usize = 40;

lazy_static! {
    pub static ref INFO: StyledObject<&'static str> = style("Info").blue().bold();
    pub static ref WARN: StyledObject<&'static str> = style("Warning").yellow().bold();
    pub static ref ERROR: StyledObject<&'static str> = style("Error").red().bold();
    pub static ref SUCCESS: StyledObject<&'static str> = style("Success").green().bold();
    pub static ref FINISHED: StyledObject<&'static str> = style("Finished").green().bold();
}

#[allow(dead_code)]
pub fn error(msg: impl AsRef<str> + Display) {
    println!("{} {}", *ERROR, msg);
}

#[allow(dead_code)]
pub fn info(msg: impl AsRef<str> + Display) {
    println!("{} {}", *INFO, msg);
}

#[allow(dead_code)]
pub fn warn(msg: impl AsRef<str> + Display) {
    println!("{} {}", *WARN, msg);
}

#[allow(dead_code)]
pub fn success(msg: impl AsRef<str> + Display) {
    println!("{} {}", *SUCCESS, msg);
}

#[allow(dead_code)]
pub fn finished(msg: &str) {
    println!("{} {}", *FINISHED, msg);
}

const ABOUT: &str = r#"  ____  _             _    _  __       
/ ___|| |_ __ _  ___| | _(_)/ _|_   _ 
\___ \| __/ _` |/ __| |/ / | |_| | | |
 ___) | || (_| | (__|   <| |  _| |_| |
|____/ \__\__,_|\___|_|\_\_|_|  \__, |
                                |___/ "#;

/// Command
#[derive(Debug, Parser)]
#[clap(
    author = "Cyle Witruk (https://github.com/cylewitruk)", 
    version,
    about = "Tooling to make it easy to run and work with local Stacks environments",
    long_about = ABOUT,
    styles=styles(),
    max_term_width = 100,
    next_line_help = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Generate completion scripts for the specified shell.
    #[clap(long, value_parser, help_heading = "Other")]
    pub completion: Option<Shell>,

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
    /// Initializes the local environment in preparation for building & running
    /// Stacks environments. This will download several binaries and build several
    /// Docker images and will take some time.
    #[clap(visible_alias = "init")]
    Initialize(InitArgs),
    /// Commands for configuring, manipulating and interacting with environments.
    #[clap(visible_alias = "env")]
    Environment(EnvArgs),
    /// Displays information about current environments and optionally other
    /// details.
    Info(InfoArgs),
    /// Cleans up resources created/used by stackify.
    Clean(CleanArgs),
    /// Commands for interacting with the stackify global configuration.
    Config(ConfigArgs),
    Completions {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Red.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}
