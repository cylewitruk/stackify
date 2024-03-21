use std::{ops::Deref, process::exit};

use bollard::container::CreateContainerOptions;
use log::debug;
use rand::{thread_rng, Rng};

use crate::{util::random_hex, EnvironmentName, StackifyDocker};

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
        debug!("Dropping test network: {}", self.0);
        let ctx = StackifyDocker::new().unwrap();
        let _ = ctx.runtime.block_on(async {
            if ctx.docker.remove_network(self)
                .await
                .is_err() { return; }
        });
        debug!("Dropped test network: {}", self.0)
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
        debug!("Dropping test container: {}", self.0);
        let ctx = StackifyDocker::new().unwrap();
        ctx.runtime.block_on(async {
            ctx.docker.remove_container(self, None)
                .await
                .expect(&format!("failed to stop container: {}", self));
        });
        debug!("Dropped test container: {}", self.0)
    }
}

fn create_test_container(docker: &StackifyDocker) -> TestContainer {
    debug!("Creating test container");

    let busybox_image = "busybox:latest";

    docker.pull_image(busybox_image);

    let random_name = thread_rng().gen::<[u8; 4]>()
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
    
    docker.runtime.block_on(async {
        docker.docker.create_container(Some(opts), config)
            .await
            .expect("failed to create test busybox container");
    });

    debug!("Created test container: {}", container_name);

    TestContainer(container_name)
}

pub fn get_docker() -> StackifyDocker {
    StackifyDocker::new()
        .map_err(|e| {
            debug!("Failed to create docker context: {}", e);
            exit(1);
        }).unwrap()
}

pub fn random_environment_name() -> EnvironmentName {
    EnvironmentName::new(&format!("test-env-{}", random_hex(4)))
        .expect("Failed to create environment name")
}