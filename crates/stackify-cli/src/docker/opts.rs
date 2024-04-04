use std::{collections::HashMap, path::Path};

use docker_api::opts::{
    ContainerCreateOpts, ContainerFilter, ContainerListOpts, ContainerStatus, ImageBuildOpts,
    NetworkCreateOpts, NetworkFilter, NetworkListOpts,
};
use stackify_common::{docker::LabelKey, types::EnvironmentName};

pub trait CreateContainer {
    fn for_stackify_build_container<P: AsRef<Path>>(
        bin_dir: &P,
        assets_dir: &P,
        env_vars: HashMap<String, String>,
    ) -> ContainerCreateOpts {
        let bin_mount = format!("{}:/out:rw", bin_dir.as_ref().to_string_lossy());

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
            .env(env_vars.iter().map(|(k, v)| format!("{}={}", k, v)))
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
    fn for_build_image<P: AsRef<Path>>(assets_dir: &P) -> ImageBuildOpts {
        let (uid, gid) = uid_gid();
        let mut build_args = HashMap::<String, String>::new();
        build_args.insert("USER_ID".into(), uid.to_string());
        build_args.insert("GROUP_ID".into(), gid.to_string());

        ImageBuildOpts::builder(assets_dir)
            .tag("stackify-build:latest")
            .labels(default_labels())
            .dockerfile("Dockerfile.build")
            .build_args(build_args)
            .build()
    }

    fn for_runtime_image<P: AsRef<Path>>(assets_dir: &P) -> ImageBuildOpts {
        let (uid, gid) = uid_gid();
        let mut build_args = HashMap::<String, String>::new();
        build_args.insert("USER_ID".into(), uid.to_string());
        build_args.insert("GROUP_ID".into(), gid.to_string());

        ImageBuildOpts::builder(assets_dir)
            .tag("stackify-runtime:latest")
            .labels(default_labels())
            .dockerfile("Dockerfile.runtime")
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
