use std::fmt::Display;
use std::path::PathBuf;

use clap::builder::styling::{Effects, Styles};
use clap::{command, Parser, Subcommand};
use clap_complete::Shell;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::Result;
//use owo_colors::OwoColorize;

use crate::cli::theme::ThemedObject;

use self::clean::CleanArgs;
use self::config::args::ConfigArgs;
use self::env::args::EnvArgs;
use self::info::InfoArgs;
use self::install::InitArgs;
use self::theme::THEME;

// Top-level command handlers
pub mod clean;
pub mod config;
pub mod context;
pub mod env;
pub mod info;
pub mod install;
pub mod log;
pub mod network;
pub mod show;
pub mod theme;
pub mod uninstall;

pub mod clap_color_flag;
pub mod clap_verbosity_flag;

#[allow(dead_code)]
pub fn error(msg: impl AsRef<str> + Display) {
    println!("{} {}", "Error:".error().bold(), msg);
}

#[allow(dead_code)]
pub fn info(msg: impl AsRef<str> + Display) {
    println!("{} {}", "Info:".info().bold(), msg);
}

#[allow(dead_code)]
pub fn warn(msg: impl AsRef<str> + Display) {
    println!("{} {}", "Warning:".warning().bold(), msg);
}

#[allow(dead_code)]
pub fn success(msg: impl AsRef<str> + Display) {
    println!("{} {}", "Success:".success().bold(), msg);
}

#[allow(dead_code)]
pub fn finished(msg: &str) {
    println!("{} {}", "Finished:".success().bold(), msg);
}

const ABOUT: &str = r#" ____  _             _    _  __       
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
    //next_line_help = true
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

    #[arg(
        long = "dump-logs",
        default_value = "false",
        hide = true,
        global = true
    )]
    pub dump_logs: bool,
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
    Uninstall,
    Completions {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
    #[clap(hide = true)]
    MarkdownHelp,
}

fn styles() -> Styles {
    // Red: 204, 55, 46
    // Green: 38, 164, 57
    // Yellow: 205, 172, 8
    // Blue: 8, 105, 203
    // Magenta: 150, 71, 191
    // Cyan: 71, 158, 194
    // Gray: 152, 152, 157
    Styles::styled()
        .header(
            owo_to_anstyle_color(THEME.read().unwrap().palette().green).on_default()
                | Effects::BOLD,
        )
        .usage(
            owo_to_anstyle_color(THEME.read().unwrap().palette().green).on_default()
                | Effects::BOLD,
        )
        .literal(owo_to_anstyle_color(THEME.read().unwrap().palette().cyan).on_default())
        .invalid(owo_to_anstyle_color(THEME.read().unwrap().palette().red).on_default())
        .valid(owo_to_anstyle_color(THEME.read().unwrap().palette().green).on_default())
        .error(
            owo_to_anstyle_color(THEME.read().unwrap().palette().red).on_default() | Effects::BOLD,
        )
        .placeholder(owo_to_anstyle_color(THEME.read().unwrap().palette().cyan).on_default())
}

fn owo_to_anstyle_color(color: owo_colors::Rgb) -> anstyle::RgbColor {
    anstyle::RgbColor(color.0, color.1, color.2)
}

#[derive(Debug, Clone)]
pub struct StackifyHostDirs {
    pub app_root: PathBuf,
    /// The local directory where Stackify binaries are stored. This includes
    /// built artifacts which are mounted to containers.
    /// Default: `~/.stackify/bin/`.
    pub bin_dir: PathBuf,
    /// The local directory where Stackify temporary data is stored.
    /// Default: `~/.stackify/tmp/`.
    pub tmp_dir: PathBuf,
    /// The local directory where Stackify assets are stored. These are additional
    /// files such as configuration file templates, shell scripts, etc.
    /// Default: `~/.stackify/assets/`.
    pub assets_dir: PathBuf,
}

impl Default for StackifyHostDirs {
    fn default() -> Self {
        let home_dir = home::home_dir().unwrap();

        Self {
            app_root: home_dir.join(".stackify"),
            bin_dir: home_dir.join(".stackify/bin"),
            tmp_dir: home_dir.join(".stackify/tmp"),
            assets_dir: home_dir.join(".stackify/assets"),
        }
    }
}
