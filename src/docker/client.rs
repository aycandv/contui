use std::sync::Arc;

use bollard::Docker;
use tracing::{debug, info};

use crate::core::{ConnectionInfo, DockerError, Result};

/// Docker client wrapper
#[derive(Clone)]
pub struct DockerClient {
    inner: Arc<Docker>,
    connection_info: ConnectionInfo,
}

impl DockerClient {
    /// Create a new client from environment (DOCKER_HOST, etc.)
    pub async fn from_env() -> Result<Self> {
        info!("Creating Docker client from environment");
        
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| DockerError::Connection(e.to_string()))?;
        
        Self::new(docker).await
    }

    /// Create a new client with custom host
    pub async fn with_host(host: &str) -> Result<Self> {
        info!("Creating Docker client with host: {}", host);
        
        let docker = Docker::connect_with_http(host, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e| DockerError::Connection(e.to_string()))?;
        
        Self::new(docker).await
    }

    /// Internal constructor
    async fn new(docker: Docker) -> Result<Self> {
        debug!("Fetching Docker version information");
        
        let version = docker.version().await
            .map_err(|e| DockerError::Connection(e.to_string()))?;
        
        let info = ConnectionInfo {
            host: "local".to_string(), // Docker doesn't expose host URL directly
            version: version.version.unwrap_or_else(|| "unknown".to_string()),
            api_version: version.api_version.unwrap_or_else(|| "unknown".to_string()),
            os: version.os.unwrap_or_else(|| "unknown".to_string()),
            arch: version.arch.unwrap_or_else(|| "unknown".to_string()),
        };
        
        info!(
            "Docker client initialized: {} (API: {}) on {}/{}",
            info.version, info.api_version, info.os, info.arch
        );
        
        Ok(Self {
            inner: Arc::new(docker),
            connection_info: info,
        })
    }

    /// Get connection information
    pub fn connection_info(&self) -> &ConnectionInfo {
        &self.connection_info
    }

    /// Ping the Docker daemon
    pub async fn ping(&self) -> Result<String> {
        debug!("Pinging Docker daemon");
        
        let response = self.inner.ping().await
            .map_err(|e| DockerError::Connection(e.to_string()))?;
        
        Ok(response)
    }

    /// Get the inner Docker client (for advanced usage)
    pub fn inner(&self) -> &Docker {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require Docker to be running
    // Mark them with #[ignore] for CI environments without Docker

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn test_from_env() {
        let client = DockerClient::from_env().await;
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert!(!client.connection_info().version.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn test_ping() {
        let client = DockerClient::from_env().await.unwrap();
        let result = client.ping().await;
        assert!(result.is_ok());
    }
}
