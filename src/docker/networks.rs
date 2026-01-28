//! Network operations

use bollard::network::ListNetworksOptions;
use tracing::{debug, info};

use crate::core::{DockerError, NetworkScope, NetworkSummary, Result};
use crate::docker::DockerClient;

impl DockerClient {
    /// List all networks
    pub async fn list_networks(&self) -> Result<Vec<NetworkSummary>> {
        debug!("Listing networks");

        let options = ListNetworksOptions::<String> {
            filters: Default::default(),
        };

        let networks = self
            .inner()
            .list_networks(Some(options))
            .await
            .map_err(|e| DockerError::Network(e.to_string()))?;

        info!("Found {} networks", networks.len());

        Ok(networks.into_iter().map(|n| n.into()).collect())
    }

    /// Remove a network
    pub async fn remove_network(&self, id: &str) -> Result<()> {
        info!("Removing network: {}", id);

        self.inner()
            .remove_network(id)
            .await
            .map_err(|e| DockerError::Network(format!("Failed to remove {}: {}", id, e)))?;

        info!("Network {} removed successfully", id);
        Ok(())
    }

    /// Prune unused networks
    pub async fn prune_networks(&self) -> Result<u64> {
        info!("Pruning unused networks");

        let result = self
            .inner()
            .prune_networks::<String>(None)
            .await
            .map_err(|e| DockerError::Network(format!("Failed to prune networks: {}", e)))?;

        let count = result.networks_deleted.map(|n| n.len() as u64).unwrap_or(0);
        info!("Pruned {} networks", count);
        Ok(count)
    }
}

impl From<bollard::models::Network> for NetworkSummary {
    fn from(n: bollard::models::Network) -> Self {
        // Parse scope
        let scope = match n.scope.as_deref() {
            Some("global") => NetworkScope::Global,
            Some("swarm") => NetworkScope::Swarm,
            _ => NetworkScope::Local,
        };

        // Get subnet from IPAM config if available
        let _subnet = n
            .ipam
            .as_ref()
            .and_then(|ipam| ipam.config.as_ref())
            .and_then(|configs| configs.first())
            .and_then(|config| config.subnet.clone())
            .unwrap_or_else(|| "-".to_string());

        // Get connected containers
        let connected_containers = n
            .containers
            .map(|c| c.into_keys().collect())
            .unwrap_or_default();

        Self {
            id: n.id.clone().unwrap_or_default(),
            name: n.name.clone().unwrap_or_default(),
            driver: n.driver.clone().unwrap_or_else(|| "bridge".to_string()),
            scope,
            created: n.created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            internal: n.internal.unwrap_or(false),
            attachable: n.attachable.unwrap_or(false),
            ingress: n.ingress.unwrap_or(false),
            enable_ipv6: n.enable_ipv6.unwrap_or(false),
            connected_containers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require Docker to be running

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn test_list_networks() {
        let client = DockerClient::from_env().await.unwrap();
        let networks = client.list_networks().await;
        assert!(networks.is_ok());
    }
}
