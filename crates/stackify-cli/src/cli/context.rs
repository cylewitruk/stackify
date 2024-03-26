use std::path::PathBuf;
use stackify_common::docker::stackify_docker::StackifyDocker;

use crate::db::AppDb;

pub struct CliContext {
    /// The configuration directory for Stackify. Defaults to `$HOME/.stackify`
    pub config_dir: PathBuf,
    /// The configuration file for Stackify. Defaults to
    /// `$HOME/.stackify/config.toml`
    pub config_file: PathBuf,
    /// The directory where Stackify stores environment data. Defaults to
    /// `$HOME/.stackify/data`
    pub data_dir: PathBuf,
    /// The directory where Stackify stores binaries. Defaults to
    /// `$HOME/.stackify/bin`
    pub bin_dir: PathBuf,
    /// The temporary directory for Stackify. Defaults to `/tmp/stackify`.
    pub tmp_dir: PathBuf,
    /// The database file for Stackify. Defaults to `$HOME/.stackify/stackify.db`
    pub db_file: PathBuf,
    /// Instance of Stackify's application database.
    pub db: AppDb,
    /// The user id of the current user.
    pub user_id: u32,
    /// The group id of the current user.
    pub group_id: u32,
    /// Instance of Stackify's Docker client.
    pub docker: StackifyDocker,
}