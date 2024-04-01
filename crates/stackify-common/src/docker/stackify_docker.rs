use std::path::Path;
use std::path::PathBuf;

use bollard::{Docker, API_DEFAULT_VERSION};

use color_eyre::eyre::{eyre, Result};
use tokio::runtime::Runtime;

use super::DockerVersion;

/// A Docker client for Stackify which also includes a Tokio runtime for
/// sync-wrapping async functions.
///
/// This struct is the primary interface for interacting with Docker in the
/// Stackify CLI and Daemon.
#[allow(dead_code)]
pub struct StackifyDocker {
    pub(crate) docker: bollard::Docker,
    pub(crate) runtime: Runtime,
    pub(crate) user_id: u32,
    pub(crate) group_id: u32,
    pub(crate) stackify_root_dir: PathBuf,
    pub(crate) stackify_bin_dir: PathBuf,
    pub(crate) stackify_data_dir: PathBuf,
    pub(crate) stackify_config_dir: PathBuf,
}

impl Default for StackifyDocker {
    fn default() -> Self {
        StackifyDocker::new().expect("Failed to connect to the Docker daemon.")
    }
}

impl StackifyDocker {
    /// Creates a new instance of `StackifyDocker`, attempting to connect to
    /// the Docker daemon. It will attempt several connection paradigms, including
    /// the new 'rootless' model on Unix systems.
    pub fn new() -> Result<Self> {
        let uid;
        let gid;
        unsafe {
            uid = libc::geteuid();
            gid = libc::getegid();
        }

        let mut docker: Option<Docker>;

        #[cfg(unix)]
        {
            docker = if Path::new("~/.docker/run/docker.sock").exists() {
                Some(Docker::connect_with_socket(
                    "~/.docker/run/docker.sock",
                    3,
                    API_DEFAULT_VERSION,
                )?)
            } else if Path::new(&format!("/run/user/{}/docker.sock", uid)).exists() {
                Some(Docker::connect_with_socket(
                    &format!("/run/user/{}/docker.sock", uid),
                    3,
                    API_DEFAULT_VERSION,
                )?)
            } else {
                if let Ok(docker_host) = std::env::var("DOCKER_HOST") {
                    Some(Docker::connect_with_socket(
                        &docker_host,
                        3,
                        API_DEFAULT_VERSION,
                    )?)
                } else {
                    None
                }
            };
        }

        if docker.is_none() {
            docker = Some(Docker::connect_with_defaults()?);
        }

        if docker.is_none() {
            return Err(eyre!("Failed to connect to the Docker daemon."));
        }

        let stackify_root_dir = PathBuf::from("/stackify");

        Ok(StackifyDocker {
            docker: docker.unwrap(),
            runtime: Runtime::new()?,
            user_id: uid,
            group_id: gid,
            stackify_root_dir: stackify_root_dir.clone(),
            stackify_bin_dir: stackify_root_dir.join("bin"),
            stackify_data_dir: stackify_root_dir.join("data"),
            stackify_config_dir: stackify_root_dir.join("config"),
        })
    }
}

impl StackifyDocker {
    /// Retrieves the version of the currently running Docker daemon.
    pub fn get_docker_version(&self) -> Result<DockerVersion> {
        self.runtime.block_on(async {
            let version = self.docker.version().await?;
            let ret = DockerVersion {
                version: version.version.unwrap_or("--".to_string()),
                api_version: version.api_version.unwrap_or("--".to_string()),
                min_api_version: version.min_api_version.unwrap_or("--".to_string()),
                components: version
                    .components
                    .map(|comp| {
                        comp.iter()
                            .map(|c| format!("{}: {}", c.name, c.version))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            };

            Ok(ret)
        })
    }
}
