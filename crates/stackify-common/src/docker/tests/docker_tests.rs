use std::path::Path;

use super::{create_test_container, get_docker, random_environment_name};

#[tokio::test]
pub async fn can_upload_ephemeral_file_to_container() {
    let docker = get_docker();

    let destination_path = Path::new("/root/test.txt");
    let data = b"Hello, World!";

    let container_name = &create_test_container(&docker).await;

    docker
        .upload_ephemeral_file_to_container(container_name, destination_path, data)
        .await
        .expect("failed to upload file to container");
}

#[tokio::test]
pub async fn can_create_and_drop_network() {
    let docker = get_docker();

    let env_name = random_environment_name();
    let network_name = "stackify-".to_string() + env_name.as_str();

    let result = docker
        .create_stackify_network(&env_name)
        .await
        .expect("Failed to create network");

    assert_eq!(result.name, network_name);
    assert!(result.id.len() > 0);

    let networks = docker
        .list_stacks_networks()
        .await
        .expect("Failed to list networks")
        .iter()
        .map(|n| n.name.clone())
        .collect::<Vec<_>>();

    assert!(networks.contains(&network_name));

    docker
        .rm_stacks_network(&env_name)
        .await
        .expect("Failed to drop network");
}

#[tokio::test]
pub async fn can_delete_all_stackify_networks() {
    let docker = get_docker();

    for _ in 0..5 {
        let env_name = random_environment_name();
        docker
            .create_stackify_network(&env_name)
            .await
            .expect("Failed to create network");
    }

    let count = docker
        .list_stacks_networks()
        .await
        .expect("Failed to list networks")
        .len();

    assert_eq!(count, 5);

    docker
        .rm_all_stacks_networks()
        .await
        .expect("Failed to drop all networks");

    let count = docker
        .list_stacks_networks()
        .await
        .expect("Failed to list networks")
        .len();

    assert_eq!(count, 0);
}
