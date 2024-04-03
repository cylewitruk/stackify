use std::{collections::HashMap, path::Path};

use docker_api::opts::{
    ContainerCreateOpts, ContainerFilter, ContainerListOpts, ContainerStatus, NetworkCreateOpts,
    NetworkFilter, NetworkListOpts,
};
use stackify_common::{docker::LabelKey, types::EnvironmentName};

pub trait CreateContainer {
    fn for_stackify_build_container<P: AsRef<Path>>(
        bin_dir: &P,
        assets_dir: &P,
    ) -> ContainerCreateOpts {
        let bin_mount = format!("{}/bin:/out:rw", bin_dir.as_ref().to_string_lossy());

        let entrypoint_mount = format!(
            "{}/build-entrypoint.sh:/entrypoint.sh",
            assets_dir.as_ref().to_string_lossy()
        );

        let mut labels = HashMap::new();
        labels.insert(LabelKey::Stackify.to_string(), "");

        ContainerCreateOpts::builder()
            .name("stackify-build")
            .volumes([bin_mount, entrypoint_mount])
            .entrypoint(["/bin/sh", "/entrypoint.sh"])
            .image("stackify-build:latest")
            .labels(labels)
            .build()
    }

    fn for_stackify_environment_container(
        environment_name: &EnvironmentName,
    ) -> ContainerCreateOpts {
        let mut labels = HashMap::new();
        labels.insert(LabelKey::Stackify.to_string(), "");
        labels.insert(LabelKey::EnvironmentName.to_string(), environment_name);

        ContainerCreateOpts::builder()
            .name(environment_name.to_string())
            .labels(labels)
            .build()
    }
}

impl CreateContainer for ContainerCreateOpts {}

pub trait CreateNetwork {
    fn for_stackify_environment(environment_name: &EnvironmentName) -> NetworkCreateOpts {
        NetworkCreateOpts::builder(environment_name.to_string()).build()
    }
}

impl CreateNetwork for NetworkCreateOpts {}

pub trait ListContainers {
    fn for_all_stackify_containers() -> ContainerListOpts {
        ContainerListOpts::builder()
            .filter([ContainerFilter::LabelKey(LabelKey::Stackify.into())])
            .build()
    }

    fn running_in_environment(environment_name: &EnvironmentName) -> ContainerListOpts {
        ContainerListOpts::builder()
            .filter([
                ContainerFilter::LabelKey(format!("label={}", LabelKey::Stackify)),
                ContainerFilter::Label(LabelKey::EnvironmentName.into(), environment_name.into()),
                ContainerFilter::Status(ContainerStatus::Running),
            ])
            .build()
    }

    fn for_environment(environment_name: &EnvironmentName) -> ContainerListOpts {
        ContainerListOpts::builder()
            .filter([
                ContainerFilter::LabelKey(format!("label={}", LabelKey::Stackify)),
                ContainerFilter::Label(LabelKey::EnvironmentName.into(), environment_name.into()),
            ])
            .build()
    }
}

impl ListContainers for ContainerListOpts {}

pub trait ListNetworks {
    fn for_all_stackify_networks() -> NetworkListOpts {
        NetworkListOpts::builder()
            .filter([NetworkFilter::LabelKey(LabelKey::Stackify.into())])
            .build()
    }
}

impl ListNetworks for NetworkListOpts {}
