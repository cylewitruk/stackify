#![allow(dead_code)]

use color_eyre::{eyre::eyre, Result};
use docker_api::{
    models::{ContainerSummary, Network},
    opts::{ContainerFilter, ContainerListOpts, NetworkCreateOpts, NetworkFilter, NetworkListOpts},
    Id,
};

use stackify_common::docker::{LabelKey, StackifyNetwork};

use super::{StackifyContainerDirs, StackifyHostDirs};

pub struct DockerApi {
    docker: ::docker_api::Docker,
    host_dirs: StackifyHostDirs,
    container_dirs: StackifyContainerDirs,
}

impl DockerApi {
    pub fn new(host_dirs: StackifyHostDirs, container_dirs: StackifyContainerDirs) -> Result<Self> {
        Ok(Self {
            docker: ::docker_api::Docker::new("tcp://127.0.0.1:2375")?,
            host_dirs,
            container_dirs,
        })
    }

    pub fn api(&self) -> &::docker_api::Docker {
        &self.docker
    }
}

impl DockerApi {
    pub async fn create_network(&self, network_name: &str) -> Result<StackifyNetwork> {
        let opts = NetworkCreateOpts::builder(network_name).build();

        let result = self.docker.networks().create(&opts).await?;

        Ok(StackifyNetwork {
            id: result.id().to_string(),
            name: network_name.to_string(),
        })
    }

    pub async fn delete_network(&self, network_name: &str) -> Result<()> {
        self.docker.networks().get(network_name).delete().await?;

        Ok(())
    }

    pub async fn start_container(&self, container_name: &str) -> Result<()> {
        let (container_id, _) = self
            .find_container_by_name(container_name)
            .await?
            .ok_or_else(|| eyre!("Container '{}' not found.", container_name))?;

        self.docker.containers().get(container_id).start().await?;

        Ok(())
    }

    pub async fn delete_container(&self, container_name: &str) -> Result<()> {
        let (container_id, _) = self
            .find_container_by_name(container_name)
            .await?
            .ok_or_else(|| eyre!("Container '{}' not found.", container_name))?;

        self.docker.containers().get(container_id).delete().await?;

        Ok(())
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
                ContainerFilter::Name(container_name.to_string()),
                ContainerFilter::LabelKey(format!("label={}", LabelKey::Stackify)),
            ])
            .build();

        let containers = self.docker.containers().list(&list_opts).await?;

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
    async fn find_network_by_name(&self, network_name: &str) -> Result<Option<(Id, Network)>> {
        let list_opts = NetworkListOpts::builder()
            .filter([
                NetworkFilter::Name(network_name.into()),
                NetworkFilter::LabelKey(format!("label={}", LabelKey::Stackify)),
            ])
            .build();

        let networks = self.docker.networks().list(&list_opts).await?;

        if networks.len() == 0 {
            return Ok(None);
        }

        if networks.len() > 1 {
            return Err(eyre!(
                "Found more than one network with the name '{}'. This shouldn't be able to happen.",
                network_name
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
