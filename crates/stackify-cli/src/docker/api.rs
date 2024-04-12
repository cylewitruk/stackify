#![allow(dead_code)]

use std::collections::HashMap;

use color_eyre::{eyre::eyre, owo_colors::OwoColorize, Result};
use docker_api::{
    models::{ContainerSummary, Network},
    opts::{
        ContainerCreateOpts, ContainerFilter, ContainerListOpts, NetworkFilter, NetworkListOpts,
    },
    Id,
};

use stackify_common::{
    types::{EnvironmentName, EnvironmentService},
    ServiceType,
};

use crate::{
    cli::{log::clilog, StackifyHostDirs},
    docker::LabelKey,
    util::names::service_container_name,
};

use super::{format_network_name, ContainerUser, StackifyContainerDirs};

#[derive(Clone)]
pub struct DockerApi {
    docker: ::docker_api::Docker,
    host_dirs: StackifyHostDirs,
    container_dirs: StackifyContainerDirs,
    container_user: ContainerUser,
    rootless: bool,
}

impl DockerApi {
    pub async fn new(
        host_dirs: StackifyHostDirs,
        container_dirs: StackifyContainerDirs,
    ) -> Result<Self> {
        let docker = docker_api::Docker::new("unix:///var/run/user/1000/docker.sock")?;

        let docker_info = docker.info().await?;
        let rootless = docker_info.security_options.map_or(false, |sec_opts| {
            sec_opts.iter().any(|opt| opt == "name=rootless")
        });

        let container_user = if rootless {
            println!(
                "{} {}",
                "Note:".dimmed().bold(),
                "Docker is running in rootless mode, using root user for containers."
            );
            ContainerUser::root()
        } else {
            unsafe { ContainerUser::new(libc::geteuid(), libc::getegid()) }
        };

        Ok(Self {
            docker,
            host_dirs,
            container_dirs,
            container_user,
            rootless,
        })
    }

    pub fn api(&self) -> &::docker_api::Docker {
        &self.docker
    }

    pub fn container_dirs(&self) -> &StackifyContainerDirs {
        &self.container_dirs
    }

    pub fn user(&self) -> &ContainerUser {
        &self.container_user
    }
}

impl DockerApi {
    pub fn opts_for(&self) -> DockerOptsHelper {
        DockerOptsHelper::new(self)
    }

    /// Helper function to find a container and its id from name, so that the
    /// rest of the application can refer to containers by name.
    ///
    /// Returns [`None`] if no container is found, and an error if more than one
    /// container is found (which should not be possible). Otherwise this function
    /// returnes a tuple of the container id and the container summary.
    pub async fn find_container_by_name(
        &self,
        container_name: &str,
    ) -> Result<Option<(Id, ContainerSummary)>> {
        let list_opts = ContainerListOpts::builder()
            .filter([
                ContainerFilter::Name(format!("^/{}$", container_name.to_string())),
                ContainerFilter::LabelKey(LabelKey::Stackify.into()),
            ])
            .all(true)
            .build();

        let containers = self.docker.containers().list(&list_opts).await?;

        clilog!(
            "Found {} containers with the name '{}'",
            containers.len(),
            container_name
        );

        if containers.len() == 0 {
            return Ok(None);
        }
        if containers.len() > 1 {
            return Err(eyre!("Found more than one container with the name '{}'. This shouldn't be able to happen.", container_name));
        }

        let container = &containers[0];
        let container_id = container
            .id
            .clone()
            .ok_or(eyre!("Container '{}' has no id.", container_name))?;

        Ok(Some((container_id.into(), container.clone())))
    }

    /// Helper function to find a network and its id from name, so that the
    /// rest of the application can refer to networks by name.
    ///
    /// Returns [`None`]
    /// if no network is found, and an error if more than one network is found
    /// (which should not be possible). Otherwise this function returns a tuple
    /// of the network id and the network summary.
    pub async fn find_network_for_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Option<(Id, Network)>> {
        let network_name = format_network_name(env_name);

        let list_opts = NetworkListOpts::builder()
            .filter([
                NetworkFilter::Name(network_name.clone()),
                NetworkFilter::LabelKey(LabelKey::Stackify.into()),
            ])
            .build();

        let networks = self.docker.networks().list(&list_opts).await?;

        if networks.len() == 0 {
            return Ok(None);
        }

        if networks.len() > 1 {
            return Err(eyre!(
                "Found more than one network with the name '{}'. This shouldn't be able to happen.",
                network_name.clone()
            ));
        }

        let network = &networks[0];
        let network_id = network
            .id
            .clone()
            .ok_or(eyre!("Network '{}' has no id.", network_name))?;

        Ok(Some((network_id.into(), network.clone())))
    }
}

pub struct DockerOptsHelper<'a>(&'a DockerApi);

impl<'a> DockerOptsHelper<'a> {
    fn new(api: &'a DockerApi) -> Self {
        Self(api)
    }

    pub fn generate_stacks_keychain(&self) -> ContainerCreateOpts {
        let labels = default_labels(None, None);

        ContainerCreateOpts::builder()
            .name("stx-stacks-cli")
            .command(["npx", "@stacks/cli", "make_keychain"])
            .attach_stdout(true)
            .auto_remove(true)
            .image("stacks-cli:latest")
            .labels(labels)
            .build()
    }

    pub fn create_bitcoin_container(
        &self,
        env_name: &EnvironmentName,
        service: &EnvironmentService,
    ) -> Result<ContainerCreateOpts> {
        let labels = default_labels(Some(env_name), Some(service));

        let bin_mount = format!(
            "{}:/opt/stackify/bin:rw",
            self.0.host_dirs.bin_dir.to_string_lossy()
        );

        let entrypoint_mount = format!(
            "{}:/entrypoint.sh:ro",
            self.0
                .host_dirs
                .assets_dir
                .join("bitcoin-entrypoint.sh")
                .to_string_lossy()
        );

        let is_miner = ServiceType::from_i32(service.service_type.id)? == ServiceType::BitcoinMiner;

        let opts = ContainerCreateOpts::builder()
            .name(service_container_name(service))
            .hostname(&service.name)
            .user(self.0.container_user.to_string())
            .volumes([bin_mount, entrypoint_mount])
            .image("stackify-runtime:latest")
            .labels(labels)
            .env(vec![
                format!("BITCOIN_VERSION={}", service.version.version),
                format!("BITCOIN_MINER={is_miner}"),
            ])
            .entrypoint(["/bin/sh", "/entrypoint.sh"])
            .build();

        Ok(opts)
    }

    pub fn create_stacks_node_container(
        &self,
        env_name: &EnvironmentName,
        service: &EnvironmentService,
    ) -> Result<ContainerCreateOpts> {
        let labels = default_labels(Some(env_name), Some(service));

        let bin_mount = format!(
            "{}:/opt/stackify/bin:rw",
            self.0.host_dirs.bin_dir.to_string_lossy()
        );

        let entrypoint_mount = format!(
            "{}:/entrypoint.sh:ro",
            self.0
                .host_dirs
                .assets_dir
                .join("stacks-node-entrypoint.sh")
                .to_string_lossy()
        );

        let is_miner = ServiceType::from_i32(service.service_type.id)? == ServiceType::StacksMiner;
        let version = service
            .version
            .clone()
            .git_target
            .ok_or(eyre!("No git target found for service '{}'", service.name))?
            .target;

        let opts = ContainerCreateOpts::builder()
            .name(service_container_name(service))
            .hostname(&service.name)
            .user(self.0.container_user.to_string())
            .volumes([bin_mount, entrypoint_mount])
            .image("stackify-runtime:latest")
            .labels(labels)
            .env(vec![
                format!("VERSION={version}"),
                format!("MINER={is_miner}"),
            ])
            //.entrypoint(["/bin/sh", "/entrypoint.sh"])
            .entrypoint([
                "/bin/sh",
                "-c",
                "/entrypoint.sh 2>&1 | tee /var/log/stackify/stacks-node.log",
            ])
            .build();

        Ok(opts)
    }
}

fn default_labels(
    env_name: Option<&EnvironmentName>,
    service: Option<&EnvironmentService>,
) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    labels.insert(LabelKey::Stackify.into(), "".to_string());

    if let Some(env_name) = env_name {
        labels.insert(LabelKey::EnvironmentName.into(), env_name.into());
    }

    if let Some(service) = service {
        labels.insert(LabelKey::ServiceId.into(), service.id.to_string());
        labels.insert(
            LabelKey::ServiceType.into(),
            service.service_type.cli_name.clone(),
        );
        labels.insert(
            LabelKey::ServiceVersion.into(),
            service.version.version.clone(),
        );
    }

    labels
}
