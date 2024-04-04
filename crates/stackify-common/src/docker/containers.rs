use std::{collections::HashMap, path::Path};

use bollard::container::{Config, CreateContainerOptions};
use color_eyre::Result;
use futures_util::{Stream, StreamExt};

use crate::{docker::util::AddLabelFilter, types::EnvironmentName};

use super::{
    stackify_docker::StackifyDocker, util::make_filters, ContainerService, ContainerState,
    CreateContainerResult, LabelKey, ListStackifyContainerOpts, LogEntry, StackifyContainer,
    StackifyLabel,
};

impl StackifyDocker {
    pub async fn stream_container_logs(
        &self,
        container_id: &str,
    ) -> Result<impl Stream<Item = Result<LogEntry>> + Unpin + '_> {
        let logopts = bollard::container::LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        };

        let stream = self.docker.logs::<String>(container_id, Some(logopts));

        Ok(stream.map(|log| {
            Ok(LogEntry {
                message: log?.to_string(),
            })
        }))
    }

    pub async fn start_build_container(&self) -> Result<()> {
        self.docker
            .start_container::<String>("stackify-build", None)
            .await?;
        Ok(())
    }

    pub async fn create_stackify_build_container<P: AsRef<Path>>(
        &self,
        bin_dir: &P,
        assets_dir: &P,
        entrypoint: &[u8],
        build_sbtc: bool,
        build_clarinet: bool,
        build_stacks_node: Option<String>,
        build_stacks_signer: bool,
    ) -> Result<CreateContainerResult> {
        let container_name = "stackify-build";
        let opts = CreateContainerOptions {
            name: container_name.to_string(),
            ..Default::default()
        };

        let labels = vec![StackifyLabel(LabelKey::Stackify, String::new()).into()]
            .into_iter()
            .collect::<HashMap<String, String>>();

        let mut volumes: HashMap<String, HashMap<(), ()>> = HashMap::new();
        volumes.insert(
            format!("{}:/out:rw", bin_dir.as_ref().to_string_lossy()),
            HashMap::new(),
        );
        volumes.insert(
            format!(
                "{}:/entrypoint.sh",
                assets_dir
                    .as_ref()
                    .join("build-entrypoint.sh")
                    .to_string_lossy()
            ),
            HashMap::new(),
        );
        let mut env_vars = vec![];
        if let Some(version) = build_stacks_node {
            env_vars.push(format!("BUILD_STACKS={}", version));
        }

        let config = Config {
            image: Some("stackify-build:latest".to_string()),
            hostname: Some(container_name.to_string()),
            volumes: Some(volumes),
            entrypoint: Some(vec!["/bin/sh".into(), "/entrypoint.sh".into()]),
            labels: Some(labels),
            tty: Some(true),
            env: Some(env_vars),
            ..Default::default()
        };

        let container = self.docker.create_container(Some(opts), config).await?;

        self.upload_ephemeral_file_to_container(
            container_name,
            Path::new("/entrypoint.sh"),
            entrypoint,
        )
        .await?;

        Ok(CreateContainerResult {
            id: container.id,
            warnings: container.warnings,
        })
    }

    pub async fn find_container_by_name(
        &self,
        container_name: &str,
    ) -> Result<Option<StackifyContainer>> {
        let opts = bollard::container::ListContainersOptions {
            all: true,
            filters: make_filters(),
            ..Default::default()
        };

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
    }

    pub async fn create_environment_container(
        &self,
        environment_name: &EnvironmentName,
    ) -> Result<CreateContainerResult> {
        let container_name = format!("stackify-env-{}", environment_name);
        let opts = CreateContainerOptions {
            name: container_name.clone(),
            ..Default::default()
        };

        let labels = vec![
            StackifyLabel(LabelKey::EnvironmentName, environment_name.into()).into(),
            StackifyLabel(
                LabelKey::Service,
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

        let container = self.docker.create_container(Some(opts), config).await?;
        Ok(CreateContainerResult {
            id: container.id,
            warnings: container.warnings,
        })
    }

    pub async fn rm_container(&self, container_id: &str) -> Result<()> {
        self.docker.remove_container(container_id, None).await?;
        Ok(())
    }

    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        self.docker.stop_container(container_id, None).await?;
        Ok(())
    }

    /// Lists all containers with the label "local.stackify".
    /// By default, this method will only return RUNNING containers. To get all
    /// containers, set `only_running` to `false`.
    pub async fn list_stackify_containers(
        &self,
        args: ListStackifyContainerOpts,
    ) -> Result<Vec<StackifyContainer>> {
        let mut filters = make_filters();
        if let Some(env) = args.environment_name {
            filters.add_label_filter(LabelKey::EnvironmentName, &env.to_string());
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
    }

    pub fn create_stacks_node_container(&self, _environment_name: &EnvironmentName) -> Result<()> {
        Ok(())
    }
}
