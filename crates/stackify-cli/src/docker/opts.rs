use std::collections::HashMap;

use color_eyre::Result;
use docker_api::opts::{
    ContainerCreateOpts, ContainerFilter, ContainerListOpts, ContainerStatus, ImageBuildOpts,
    NetworkCreateOpts, NetworkFilter, NetworkListOpts,
};
use stackify_common::{
    docker::LabelKey,
    types::{EnvironmentName, EnvironmentService},
    ServiceType,
};

use crate::{
    cli::StackifyHostDirs,
    util::names::{environment_container_name, service_container_name},
};

use super::{ContainerUser, StackifyContainerDirs};

pub trait CreateContainer {
    /// Create [`ContainerCreateOpts`] for the build container.
    /// This container is used to compile Stacks binaries for services specified
    /// in environments in a repeatable and consistent way.
    fn for_stackify_build_container(
        container_user: &ContainerUser,
        host_dirs: &StackifyHostDirs,
        env_vars: HashMap<String, String>,
    ) -> ContainerCreateOpts {
        let bin_mount = format!("{}:/out:rw", host_dirs.bin_dir.to_string_lossy());

        let entrypoint_mount = format!(
            "{}:/entrypoint.sh",
            host_dirs
                .assets_dir
                .join("build-entrypoint.sh")
                .to_string_lossy()
        );

        let cargo_config_mount = format!(
            "{}:/cargo-config.toml",
            host_dirs
                .assets_dir
                .join("cargo-config.toml")
                .to_string_lossy()
        );

        let mut labels = HashMap::new();
        labels.insert(LabelKey::Stackify.to_string(), "");

        ContainerCreateOpts::builder()
            .name("stackify-build")
            .user(container_user.to_string())
            .volumes([bin_mount, entrypoint_mount, cargo_config_mount])
            .entrypoint(["/bin/sh", "/entrypoint.sh"])
            .image("stackify-build:latest")
            .labels(labels)
            .auto_remove(true)
            .env(env_vars.iter().map(|(k, v)| format!("{}={}", k, v)))
            .build()
    }

    /// Create [`ContainerCreateOpts`] for an environment container. This container
    /// is used to represent the environment itself, and is used as a "lock file"
    /// to prevent the same environment from being started multiple times,
    /// especially in the case where the app database is out-of-sync.
    fn for_stackify_environment_container(
        environment_name: &EnvironmentName,
    ) -> ContainerCreateOpts {
        let mut labels = HashMap::new();
        labels.insert(LabelKey::Stackify.to_string(), "");
        labels.insert(LabelKey::EnvironmentName.to_string(), environment_name);
        labels.insert(LabelKey::ServiceType.to_string(), "environment");

        ContainerCreateOpts::builder()
            .name(environment_container_name(environment_name))
            .image("busybox:latest")
            .labels(labels)
            .build()
    }

    /// Create [`ContainerCreateOpts`] for an environment service container.
    fn for_stackify_runtime_container(
        environment_name: &EnvironmentName,
        container_user: &ContainerUser,
        service: &EnvironmentService,
        host_dirs: &StackifyHostDirs,
        container_dirs: &StackifyContainerDirs,
    ) -> Result<ContainerCreateOpts> {
        let bin_mount = format!(
            "{}:{}:rw",
            host_dirs.bin_dir.to_string_lossy(),
            container_dirs.bin_dir.to_string_lossy()
        );

        let service_type = ServiceType::from_i32(service.service_type.id)?;

        let mut labels = HashMap::new();
        labels.insert(LabelKey::Stackify.to_string(), "");
        labels.insert(LabelKey::EnvironmentName.to_string(), environment_name);
        labels.insert(
            LabelKey::ServiceType.to_string(),
            &service.service_type.cli_name,
        );
        labels.insert(
            LabelKey::ServiceVersion.to_string(),
            &service.version.version,
        );
        let service_id = service.id.to_string();
        labels.insert(LabelKey::ServiceId.to_string(), &service_id);

        let opts = ContainerCreateOpts::builder()
            .name(service_container_name(service))
            .user(container_user.to_string())
            .volumes([bin_mount])
            .image("stackify-runtime:latest")
            .labels(labels)
            .build();

        Ok(opts)
    }
}

impl CreateContainer for ContainerCreateOpts {}

pub trait CreateNetwork {
    /// Create [`NetworkCreateOpts`] for a Stackify environment network.
    /// Each Stackify environment receives its own network to ensure that
    /// 1) the different services in the environment can communicate with eachother,
    /// and 2) the services in different environments are isolated from eachother.
    fn for_stackify_environment(environment_name: &EnvironmentName) -> NetworkCreateOpts {
        let mut labels = HashMap::new();
        labels.insert(LabelKey::Stackify.to_string(), "");
        labels.insert(LabelKey::EnvironmentName.to_string(), environment_name);

        NetworkCreateOpts::builder(format!("stx-{}", environment_name.to_string()))
            .labels(labels)
            .build()
    }
}

impl CreateNetwork for NetworkCreateOpts {}

pub trait ListContainers {
    fn for_all_stackify_containers() -> ContainerListOpts {
        ContainerListOpts::builder()
            .filter([ContainerFilter::LabelKey(LabelKey::Stackify.into())])
            .all(true)
            .build()
    }

    fn running_in_environment(environment_name: &EnvironmentName) -> ContainerListOpts {
        ContainerListOpts::builder()
            .filter([
                ContainerFilter::LabelKey(LabelKey::Stackify.into()),
                ContainerFilter::Label(LabelKey::EnvironmentName.into(), environment_name.into()),
                ContainerFilter::Status(ContainerStatus::Running),
            ])
            .build()
    }

    /// Creates a filter for all containers in a given environment, optionally
    /// including stopped containers (`all = true`).
    fn for_environment(environment_name: &EnvironmentName, all: bool) -> ContainerListOpts {
        ContainerListOpts::builder()
            .filter([
                ContainerFilter::LabelKey(LabelKey::Stackify.to_string()),
                ContainerFilter::Label(
                    LabelKey::EnvironmentName.to_string(),
                    environment_name.to_string(),
                ),
            ])
            .all(all)
            .build()
    }
}

impl ListContainers for ContainerListOpts {}

pub trait ListNetworks {
    /// Creates a filter for all networks with the Stackify label.
    fn for_all_stackify_networks() -> NetworkListOpts {
        NetworkListOpts::builder()
            .filter([NetworkFilter::LabelKey(LabelKey::Stackify.into())])
            .build()
    }

    fn for_environment(env_name: &EnvironmentName) -> NetworkListOpts {
        NetworkListOpts::builder()
            .filter([
                NetworkFilter::LabelKey(LabelKey::Stackify.into()),
                NetworkFilter::LabelKeyVal(LabelKey::EnvironmentName.into(), env_name.into()),
            ])
            .build()
    }
}

impl ListNetworks for NetworkListOpts {}

pub trait BuildImage {
    /// Create [`ImageBuildOpts`] for the build image. This image is used to compile
    /// Stacks binaries for services specified in environments in a repeatable and
    /// consistent way.
    fn for_build_image(
        host_dirs: &StackifyHostDirs,
        precompile: bool,
        force: bool,
    ) -> ImageBuildOpts {
        let (uid, gid) = uid_gid();
        let mut build_args = HashMap::<String, String>::new();
        build_args.insert("USER_ID".into(), uid.to_string());
        build_args.insert("GROUP_ID".into(), gid.to_string());
        build_args.insert("PRE_COMPILED".into(), precompile.to_string());

        ImageBuildOpts::builder(&host_dirs.assets_dir)
            .tag("stackify-build:latest")
            .labels(default_labels())
            .dockerfile("Dockerfile.build")
            .nocahe(force)
            .build_args(build_args)
            .build()
    }

    /// Create [`ImageBuildOpts`] for the runtime image. This image is used to run
    /// environment services in a repeatable and consistent OS environment.
    fn for_runtime_image(host_dirs: &StackifyHostDirs, force: bool) -> ImageBuildOpts {
        let (uid, gid) = uid_gid();
        let mut build_args = HashMap::<String, String>::new();
        build_args.insert("USER_ID".into(), uid.to_string());
        build_args.insert("GROUP_ID".into(), gid.to_string());

        ImageBuildOpts::builder(&host_dirs.assets_dir)
            .tag("stackify-runtime:latest")
            .labels(default_labels())
            .dockerfile("Dockerfile.runtime")
            .nocahe(force)
            .build_args(build_args)
            .build()
    }
}

impl BuildImage for ImageBuildOpts {}

/// Helper function to create the default labels for a Stackify container.
pub fn default_labels() -> HashMap<String, String> {
    let mut labels = HashMap::new();
    labels.insert(LabelKey::Stackify.to_string(), String::new());
    labels
}

/// Helper function to get the current user and group IDs.
pub fn uid_gid() -> (u32, u32) {
    let uid;
    let gid;
    unsafe {
        uid = libc::geteuid();
        gid = libc::getegid();
    }
    (uid, gid)
}
