use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use bollard::container::{Config, CreateContainerOptions};
use bollard::{
    container::UploadToContainerOptions,
    network::{CreateNetworkOptions, ListNetworksOptions},
    secret::Ipam,
    Docker, API_DEFAULT_VERSION,
};
use bytes::Bytes;
use color_eyre::eyre::{eyre, Result};
use futures_util::{Stream, TryStreamExt};
use log::debug;
use rand::{thread_rng, Rng};
use tokio::runtime::Runtime;

use crate::docker::{make_filters, AddLabelFilter};
use crate::EnvironmentName;

use super::{
    ContainerService, ContainerState, CreateContainerResult, DockerNetwork, DockerVersion, Label,
    ListStackifyContainerOpts, NewStacksNetworkResult, StackifyContainer, StacksLabel,
};

/// A Docker client for Stackify which also includes a Tokio runtime for
/// sync-wrapping async functions.
///
/// This struct is the primary interface for interacting with Docker in the
/// Stackify CLI and Daemon.
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

    pub fn list_stacks_networks(&self) -> Result<Vec<DockerNetwork>> {
        let mut filters = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![Label::EnvironmentName.to_string()],
        );
        let opts = ListNetworksOptions { filters };

        self.runtime.block_on(async {
            let networks = self
                .docker
                .list_networks(Some(opts))
                .await?
                .iter()
                .map(|n| {
                    let id =
                        n.id.as_ref()
                            .ok_or_else(|| eyre!("Failed to get network ID."))?;
                    let name = n
                        .name
                        .as_ref()
                        .ok_or_else(|| eyre!("Failed to get network name."))?;
                    Ok(DockerNetwork {
                        id: id.clone(),
                        name: name.clone(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(networks)
        })
    }

    pub fn rm_stacks_network(&self, environment_name: &EnvironmentName) -> Result<()> {
        let network_name = format!("stackify-{}", environment_name);
        self.runtime.block_on(async {
            self.docker.remove_network(&network_name).await?;
            Ok(())
        })
    }

    pub fn rm_all_stacks_networks(&self) -> Result<()> {
        let networks = self.list_stacks_networks()?;
        self.runtime.block_on(async {
            for network in networks {
                self.docker.remove_network(&network.id).await?;
            }
            Ok(())
        })
    }

    pub fn rm_network(&self, network_name: &str) -> Result<()> {
        self.runtime.block_on(async {
            self.docker.remove_network(network_name).await?;
            Ok(())
        })
    }

    pub fn create_stackify_network(
        &self,
        environment_name: &EnvironmentName,
    ) -> Result<NewStacksNetworkResult> {
        let network_name = format!("stackify-{}", environment_name);
        let labels = vec![StacksLabel(Label::EnvironmentName, environment_name.into()).into()]
            .into_iter()
            .collect::<HashMap<_, _>>();

        let opts = CreateNetworkOptions {
            name: network_name.clone(),
            check_duplicate: true,
            driver: "bridge".to_string(),
            internal: false,
            attachable: true,
            ingress: false,
            ipam: Ipam::default(),
            enable_ipv6: false,
            options: Default::default(),
            labels,
        };

        self.runtime.block_on(async {
            let result = self.docker.create_network(opts).await?;
            let id = result
                .id
                .ok_or_else(|| eyre!("Failed to create network."))?;
            Ok(NewStacksNetworkResult {
                id,
                name: network_name,
            })
        })
    }

    pub fn create_stacks_node_container(&self, _environment_name: &EnvironmentName) -> Result<()> {
        Ok(())
    }

    pub fn download_file_from_container(
        &self,
        container_name: &str,
        file_path: &Path,
    ) -> Result<Vec<u8>> {
        let opts = bollard::container::DownloadFromContainerOptions {
            path: file_path.to_string_lossy().to_string(),
        };

        self.runtime.block_on(async {
            let stream = self
                .docker
                .download_from_container(container_name, Some(opts));

            let result = concat_byte_stream(&self.runtime, stream)?;
            Ok(result)
        })
    }

    pub fn upload_ephemeral_file_to_container(
        &self,
        container_name: &str,
        destination_path: &Path,
        data: &[u8],
    ) -> Result<()> {
        let file_name = destination_path
            .file_name()
            .ok_or_else(|| eyre!("Failed to get file name."))?;
        let dir = destination_path
            .parent()
            .ok_or_else(|| eyre!("Failed to get parent directory."))?;

        let mut tar = tar::Builder::new(Vec::new());
        let mut tar_header = tar::Header::new_gnu();
        tar_header.set_mode(644);
        tar_header.set_size(data.len() as u64);

        tar.append_data(&mut tar_header, Path::new(file_name), data)?;
        tar.finish()?;

        debug!("tar header: {:?}", tar_header);
        debug!("destination path: {:?}", destination_path);

        let bytes = tar.into_inner().map_err(|e| eyre!(e))?;

        let opts = UploadToContainerOptions {
            path: format!("{}", dir.display()),
            ..Default::default()
        };

        self.runtime.block_on(async {
            self.docker
                .upload_to_container(container_name, Some(opts), bytes.into())
                .await?;
            Ok(())
        })
    }

    pub fn upload_ephemeral_files_to_container(
        &self,
        container_name: &str,
        destination_dir: &Path,
        files: Vec<(&str, &[u8])>,
    ) -> Result<()> {
        let mut tar = tar::Builder::new(Vec::new());

        for (filename, data) in files {
            let mut tar_header = tar::Header::new_gnu();
            tar_header.set_mode(644);
            tar_header.set_size(data.len() as u64);
            tar.append_data(&mut tar_header, destination_dir.join(filename), data)?;
        }
        tar.finish()?;

        let bytes = tar.into_inner().map_err(|e| eyre!(e))?;

        let opts = UploadToContainerOptions {
            path: format!("{}", destination_dir.display()),
            ..Default::default()
        };

        self.runtime.block_on(async {
            self.docker
                .upload_to_container(container_name, Some(opts), bytes.into())
                .await?;
            Ok(())
        })
    }

    /// Pulls a remove image.
    pub fn pull_image(&self, image: &str) {
        let ctx = StackifyDocker::new().unwrap();

        ctx.runtime.block_on(async {
            debug!("Pulling image: {}", image);
            ctx.docker
                .create_image(
                    Some(bollard::image::CreateImageOptions {
                        from_image: image,
                        ..Default::default()
                    }),
                    None,
                    None,
                )
                .try_collect::<Vec<_>>()
                .await
                .expect("Failed to pull image");
            debug!("Pulled image: {}", image);
        });
    }
}

#[allow(dead_code)]
fn get_new_name(environment_name: &EnvironmentName) -> String {
    let random = thread_rng()
        .gen::<[u8; 32]>()
        .iter()
        .take(4)
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    format!(
        "stx-{}-{}",
        environment_name.as_ref()[0..5].to_string(),
        random.to_lowercase()
    )
}

fn concat_byte_stream<S>(rt: &Runtime, s: S) -> Result<Vec<u8>>
where
    S: Stream<Item = std::result::Result<Bytes, bollard::errors::Error>>,
{
    rt.block_on(async {
        let result = s
            .try_fold(Vec::new(), |mut acc, chunk| async move {
                acc.extend_from_slice(&chunk[..]);
                Ok(acc)
            })
            .await?;
        Ok(result)
    })
}
