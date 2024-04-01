use std::{collections::HashMap, path::Path};

use bollard::container::{Config, CreateContainerOptions};
use color_eyre::Result;

use crate::{docker::AddLabelFilter, EnvironmentName};

use super::{
    make_filters, stackify_docker::StackifyDocker, ContainerService, ContainerState,
    CreateContainerResult, Label, ListStackifyContainerOpts, StackifyContainer, StacksLabel,
};

impl StackifyDocker {
    pub fn create_stackify_build_container<P: AsRef<Path>>(
        &self,
        bin_dir: &P,
        entrypoint: &[u8],
    ) -> Result<CreateContainerResult> {
        let container_name = "stackify-build";
        let opts = CreateContainerOptions {
            name: container_name.to_string(),
            ..Default::default()
        };

        let labels = vec![StacksLabel(Label::Stackify, String::new()).into()]
            .into_iter()
            .collect::<HashMap<String, String>>();

        let mut volumes: HashMap<String, HashMap<(), ()>> = HashMap::new();
        volumes.insert(
            format!("{}:~/.stackify/bin", bin_dir.as_ref().display()),
            HashMap::new(),
        );
        volumes.insert(
            format!(
                "{}/build-entrypoint.sh:/entrypoint.sh",
                bin_dir.as_ref().display()
            ),
            HashMap::new(),
        );

        let config = Config {
            image: Some("stackify-build:latest".to_string()),
            hostname: Some(container_name.to_string()),
            volumes: Some(volumes),
            entrypoint: Some(vec!["/entrypoint.sh".into()]),
            labels: Some(labels),
            ..Default::default()
        };

        let result = self.runtime.block_on(async {
            let container = self.docker.create_container(Some(opts), config).await?;
            Ok(CreateContainerResult {
                id: container.id,
                warnings: container.warnings,
            })
        });

        self.upload_ephemeral_file_to_container(
            container_name,
            Path::new("/entrypoint.sh"),
            entrypoint,
        )?;

        result
    }

    pub fn find_container_by_name(
        &self,
        container_name: &str,
    ) -> Result<Option<StackifyContainer>> {
        let opts = bollard::container::ListContainersOptions {
            all: true,
            filters: make_filters(),
            ..Default::default()
        };

        self.runtime.block_on(async {
            let containers = self.docker.list_containers(Some(opts)).await?;
            if let Some(container) = containers.iter().find(|c| {
                c.names
                    .as_ref()
                    .map(|names| names.iter().any(|n| n == container_name))
                    .unwrap_or(false)
            }) {
                Ok(Some(StackifyContainer {
                    id: container.id.clone().unwrap(),
                    name: container.names.clone().unwrap().join(", "),
                    labels: container.labels.clone().unwrap_or_default(),
                    state: ContainerState::parse(&container.state.clone().unwrap_or_default())
                        .expect("Failed to parse container state."),
                    status_readable: container.status.clone().unwrap_or_default(),
                }))
            } else {
                Ok(None)
            }
        })
    }

    pub fn create_environment_container(
        &self,
        environment_name: &EnvironmentName,
    ) -> Result<CreateContainerResult> {
        let container_name = format!("stackify-env-{}", environment_name);
        let opts = CreateContainerOptions {
            name: container_name.clone(),
            ..Default::default()
        };

        let labels = vec![
            StacksLabel(Label::EnvironmentName, environment_name.into()).into(),
            StacksLabel(
                Label::Service,
                ContainerService::Environment.to_label_string(),
            )
            .into(),
        ]
        .into_iter()
        .collect::<HashMap<String, String>>();

        let config = Config {
            image: Some("busybox:latest".to_string()),
            hostname: Some(container_name.clone()),
            entrypoint: Some(vec![
                "/usr/bin/env sh -c 'while true; do sleep 1; done'".to_string()
            ]),
            labels: Some(labels),

            ..Default::default()
        };

        self.runtime.block_on(async {
            let container = self.docker.create_container(Some(opts), config).await?;
            Ok(CreateContainerResult {
                id: container.id,
                warnings: container.warnings,
            })
        })
    }

    pub fn rm_container(&self, container_id: &str) -> Result<()> {
        self.runtime.block_on(async {
            self.docker.remove_container(container_id, None).await?;
            Ok(())
        })
    }

    pub fn stop_container(&self, container_id: &str) -> Result<()> {
        self.runtime.block_on(async {
            self.docker.stop_container(container_id, None).await?;
            Ok(())
        })
    }

    /// Lists all containers with the label "local.stackify".
    /// By default, this method will only return RUNNING containers. To get all
    /// containers, set `only_running` to `false`.
    pub fn list_stackify_containers(
        &self,
        args: ListStackifyContainerOpts,
    ) -> Result<Vec<StackifyContainer>> {
        let mut filters = make_filters();
        if let Some(env) = args.environment_name {
            filters.add_label_filter(Label::EnvironmentName, &env.to_string());
        }

        let opts = bollard::container::ListContainersOptions {
            all: if args.only_running.is_some() {
                !args.only_running.unwrap()
            } else {
                true
            },
            //filters,
            filters,
            ..Default::default()
        };

        eprintln!("opts: {:?}", opts);

        self.runtime.block_on(async {
            let containers = self.docker.list_containers(Some(opts)).await?;
            eprintln!("containers: {:?}", containers);
            Ok(containers
                .iter()
                .map(|c| {
                    let state = ContainerState::parse(&c.state.clone().unwrap_or_default())
                        .expect("Failed to parse container state.");

                    StackifyContainer {
                        id: c.id.clone().unwrap(),
                        name: c.names.clone().unwrap().join(", "),
                        labels: c.labels.clone().unwrap_or_default(),
                        state,
                        status_readable: c.status.clone().unwrap_or_default(),
                    }
                })
                .collect::<Vec<_>>())
        })
    }
}
