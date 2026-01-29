//! Volume operations

use bollard::volume::ListVolumesOptions;
use tracing::{debug, info};

use crate::core::{DockerError, Result, VolumeScope, VolumeSummary};
use crate::docker::DockerClient;

impl DockerClient {
    /// List all volumes
    pub async fn list_volumes(&self) -> Result<Vec<VolumeSummary>> {
        debug!("Listing volumes");

        let options = ListVolumesOptions::<String> {
            filters: Default::default(),
        };

        let volumes = self
            .inner()
            .list_volumes(Some(options))
            .await
            .map_err(|e| DockerError::Volume(e.to_string()))?;

        let volume_list = volumes.volumes.unwrap_or_default();
        info!("Found {} volumes", volume_list.len());

        Ok(volume_list.into_iter().map(|v| v.into()).collect())
    }

    /// Remove a volume
    pub async fn remove_volume(&self, name: &str, force: bool) -> Result<()> {
        info!("Removing volume: {} (force={})", name, force);

        self.inner()
            .remove_volume(name, Some(bollard::volume::RemoveVolumeOptions { force }))
            .await
            .map_err(|e| DockerError::Volume(format!("Failed to remove {}: {}", name, e)))?;

        info!("Volume {} removed successfully", name);
        Ok(())
    }

    /// Prune unused volumes
    pub async fn prune_volumes(&self) -> Result<u64> {
        info!("Pruning unused volumes");

        let result = self
            .inner()
            .prune_volumes::<String>(None)
            .await
            .map_err(|e| DockerError::Volume(format!("Failed to prune volumes: {}", e)))?;

        let reclaimed = result.space_reclaimed.unwrap_or(0) as u64;
        let deleted_count = result.volumes_deleted.as_ref().map(|v| v.len()).unwrap_or(0);
        
        if deleted_count > 0 {
            info!("Pruned {} volumes, reclaimed {} bytes", deleted_count, reclaimed);
            for vol in result.volumes_deleted.unwrap_or_default() {
                info!("  Deleted volume: {}", vol);
            }
        } else {
            info!("No volumes were pruned (reclaimed: {} bytes)", reclaimed);
        }
        
        Ok(reclaimed)
    }
}

impl From<bollard::models::Volume> for VolumeSummary {
    fn from(v: bollard::models::Volume) -> Self {
        let name = v.name.clone();

        // Parse scope
        let scope = match v.scope {
            Some(bollard::models::VolumeScopeEnum::GLOBAL) => VolumeScope::Global,
            _ => VolumeScope::Local,
        };

        Self {
            name: name.clone(),
            driver: v.driver,
            mountpoint: v.mountpoint,
            created_at: v
                .created_at
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            status: std::collections::HashMap::new(), // Simplified
            labels: v.labels,
            scope,
            options: v.options,
            in_use: vec![], // Will be populated by checking containers
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require Docker to be running

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn test_list_volumes() {
        let client = DockerClient::from_env().await.unwrap();
        let volumes = client.list_volumes().await;
        assert!(volumes.is_ok());
    }
}
