use std::{ops::Deref, process::exit};

use bollard::container::CreateContainerOptions;
use log::debug;
use rand::{thread_rng, Rng};

use crate::{docker::stackify_docker::StackifyDocker, types::EnvironmentName, util::random_hex};

pub mod docker_tests;

struct TestNetwork(String);

impl Into<String> for TestNetwork {
    fn into(self) -> String {
        self.0.to_owned()
    }
}

impl Deref for TestNetwork {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TestNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Drop for TestNetwork {
    fn drop(&mut self) {
        let name = self.0.clone();
        debug!("Dropping test network: {}", name);
        let ctx = StackifyDocker::new().unwrap();
        tokio::spawn(async move {
            if ctx.docker.remove_network(&name).await.is_err() {
                return;
            }
            debug!("Dropped test network: {}", name)
        });
    }
}

struct TestContainer(String);

impl Into<String> for TestContainer {
    fn into(self) -> String {
        self.0.to_owned()
    }
}

impl Deref for TestContainer {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TestContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Drop for TestContainer {
    fn drop(&mut self) {
        let name = self.0.clone();
        debug!("Dropping test container: {}", name);
        let ctx = StackifyDocker::new().unwrap();
        tokio::spawn(async move {
            ctx.docker
                .remove_container(&name, None)
                .await
                .expect(&format!("failed to stop container: {}", name));
        });
    }
}

async fn create_test_container(docker: &StackifyDocker) -> TestContainer {
    debug!("Creating test container");

    let busybox_image = "busybox:latest";

    docker.pull_image(busybox_image);

    let random_name = thread_rng()
        .gen::<[u8; 4]>()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let container_name = "stackify-test-".to_string() + &random_name;

    let opts = CreateContainerOptions {
        name: container_name.to_string(),
        ..Default::default()
    };

    let config = bollard::container::Config {
        image: Some(busybox_image),
        entrypoint: Some(vec!["sh -c 'while : ;do sleep 1; done'"]),
        working_dir: Some("/root"),
        ..Default::default()
    };

    docker
        .docker
        .create_container(Some(opts), config)
        .await
        .expect("failed to create test busybox container");

    debug!("Created test container: {}", container_name);

    TestContainer(container_name)
}

pub fn get_docker() -> StackifyDocker {
    StackifyDocker::new()
        .map_err(|e| {
            debug!("Failed to create docker context: {}", e);
            exit(1);
        })
        .unwrap()
}

pub fn random_environment_name() -> EnvironmentName {
    EnvironmentName::new(&format!("test-env-{}", random_hex(4)))
        .expect("Failed to create environment name")
}
