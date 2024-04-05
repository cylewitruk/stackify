use std::collections::HashMap;

use docker_api::opts::{
    ContainerCreateOpts, ContainerFilter, ContainerListOpts, ContainerStatus, ImageBuildOpts,
    NetworkCreateOpts, NetworkFilter, NetworkListOpts,
};
use stackify_common::{docker::LabelKey, types::EnvironmentName};

use crate::cli::StackifyHostDirs;

use super::{ContainerUser, StackifyContainerDirs};

pub trait CreateContainer {
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

    fn for_stackify_environment_container(
        environment_name: &EnvironmentName,
        container_user: &ContainerUser,
        host_dirs: &StackifyHostDirs,
        container_dirs: &StackifyContainerDirs,
    ) -> ContainerCreateOpts {
        let bin_mount = format!(
            "{}:{}:rw",
            host_dirs.bin_dir.to_string_lossy(),
            container_dirs.bin_dir.to_string_lossy()
        );

        let mut labels = HashMap::new();
        labels.insert(LabelKey::Stackify.to_string(), "");
        labels.insert(LabelKey::EnvironmentName.to_string(), environment_name);

        ContainerCreateOpts::builder()
            .name(environment_name.to_string())
            .user(container_user.to_string())
            .volumes([bin_mount])
            .image("stackify-runtime:latest")
            .labels(labels)
            .build()
    }
}

impl CreateContainer for ContainerCreateOpts {}

pub trait CreateNetwork {
    fn for_stackify_environment(environment_name: &EnvironmentName) -> NetworkCreateOpts {
        let mut labels = HashMap::new();
        labels.insert(LabelKey::Stackify.to_string(), "");
        labels.insert(LabelKey::EnvironmentName.to_string(), environment_name);

        NetworkCreateOpts::builder(environment_name.to_string())
            .labels(labels)
            .build()
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

pub trait BuildImage {
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

fn default_labels() -> HashMap<String, String> {
    let mut labels = HashMap::new();
    labels.insert(LabelKey::Stackify.to_string(), String::new());
    labels
}

fn uid_gid() -> (u32, u32) {
    let uid;
    let gid;
    unsafe {
        uid = libc::geteuid();
        gid = libc::getegid();
    }
    (uid, gid)
}
