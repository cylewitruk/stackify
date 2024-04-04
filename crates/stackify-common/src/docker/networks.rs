use std::collections::HashMap;

use bollard::network::{CreateNetworkOptions, ListNetworksOptions};
use color_eyre::{eyre::eyre, Result};

use crate::types::EnvironmentName;

use super::{
    stackify_docker::StackifyDocker, LabelKey, NewStacksNetworkResult, StackifyLabel,
    StackifyNetwork,
};

impl StackifyDocker {
    pub async fn list_stacks_networks(&self) -> Result<Vec<StackifyNetwork>> {
        let mut filters = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![LabelKey::EnvironmentName.to_string()],
        );
        let opts = ListNetworksOptions { filters };

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
                Ok(StackifyNetwork {
                    id: id.clone(),
                    name: name.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(networks)
    }

    pub async fn rm_stacks_network(&self, environment_name: &EnvironmentName) -> Result<()> {
        let network_name = format!("stackify-{}", environment_name);
        self.docker.remove_network(&network_name).await?;
        Ok(())
    }

    pub async fn rm_all_stacks_networks(&self) -> Result<()> {
        let networks = self.list_stacks_networks().await?;
        for network in networks {
            self.docker.remove_network(&network.id).await?;
        }
        Ok(())
    }

    pub async fn rm_network(&self, network_name: &str) -> Result<()> {
        self.docker.remove_network(network_name).await?;
        Ok(())
    }

    pub async fn create_stackify_network(
        &self,
        environment_name: &EnvironmentName,
    ) -> Result<NewStacksNetworkResult> {
        let network_name = format!("stackify-{}", environment_name);
        let labels = vec![StackifyLabel(LabelKey::EnvironmentName, environment_name.into()).into()]
            .into_iter()
            .collect::<HashMap<_, _>>();

        let opts = CreateNetworkOptions {
            name: network_name.clone(),
            check_duplicate: true,
            driver: "bridge".to_string(),
            internal: false,
            attachable: true,
            ingress: false,
            enable_ipv6: false,
            labels,
            ..Default::default()
        };

        let result = self.docker.create_network(opts).await?;
        let id = result
            .id
            .ok_or_else(|| eyre!("Failed to create network."))?;
        Ok(NewStacksNetworkResult {
            id,
            name: network_name,
        })
    }
}
