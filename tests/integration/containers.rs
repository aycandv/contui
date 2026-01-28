//! Container operations integration tests

use dockmon::docker::DockerClient;

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn test_container_lifecycle() {
    let client = DockerClient::from_env().await.unwrap();

    // List containers (should not fail even if empty)
    let containers = client.list_containers(true).await;
    assert!(containers.is_ok());

    // Note: To test start/stop/restart etc., we would need to:
    // 1. Pull a test image
    // 2. Create a container
    // 3. Test operations
    // 4. Clean up
    // 
    // For now, we just verify the methods exist and can be called
    // without panicking (they may fail if container doesn't exist)
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn test_list_running_containers() {
    let client = DockerClient::from_env().await.unwrap();

    // List only running containers
    let containers = client.list_containers(false).await;
    assert!(containers.is_ok());

    let containers = containers.unwrap();
    // All returned containers should be running
    for container in &containers {
        assert_eq!(
            container.state,
            dockmon::core::ContainerState::Running,
            "Container {} should be running",
            container.id
        );
    }
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn test_list_all_containers() {
    let client = DockerClient::from_env().await.unwrap();

    // List all containers including stopped
    let containers = client.list_containers(true).await;
    assert!(containers.is_ok());

    // We should get some result (may be empty, but shouldn't error)
    let containers = containers.unwrap();
    println!("Found {} containers", containers.len());

    // Verify container fields are populated
    for container in &containers {
        // ID should be non-empty
        assert!(!container.id.is_empty(), "Container ID should not be empty");

        // Short ID should be 12 characters
        assert_eq!(container.short_id.len(), 12);

        // Created time should be valid
        assert!(container.created.timestamp() > 0);
    }
}
