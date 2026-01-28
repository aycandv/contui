//! Integration tests for Docker client

// These tests require Docker to be running
// Use `cargo test --test integration -- --ignored` to run them

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn test_docker_connection() {
    use dockmon::docker::DockerClient;
    
    let client = DockerClient::from_env().await;
    assert!(client.is_ok());
}

#[tokio::test]
#[ignore = "requires Docker daemon"]
async fn test_list_containers() {
    use dockmon::docker::DockerClient;
    
    let client = DockerClient::from_env().await.unwrap();
    // Just verify it doesn't panic - actual list operations in US-004
    let _ = client.ping().await;
}
