//! Docker system operations (disk usage, prune)

use bollard::models::SystemDataUsageResponse;
use serde::{Deserialize, Serialize};

/// Disk usage information for a specific resource type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Total size in bytes
    pub total: i64,
    /// Reclaimable size in bytes (can be freed)
    pub reclaimable: i64,
    /// Number of items
    pub count: i64,
}

impl ResourceUsage {
    /// Create empty usage
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if there's any usage
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }
}

/// System-wide disk usage information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemDiskUsage {
    /// Image usage
    pub images: ResourceUsage,
    /// Container usage
    pub containers: ResourceUsage,
    /// Volume usage
    pub volumes: ResourceUsage,
    /// Build cache usage
    pub build_cache: ResourceUsage,
}

impl SystemDiskUsage {
    /// Get total size across all categories
    pub fn total_size(&self) -> i64 {
        self.images.total + self.containers.total + self.volumes.total + self.build_cache.total
    }

    /// Get total reclaimable size across all categories
    pub fn total_reclaimable(&self) -> i64 {
        self.images.reclaimable
            + self.containers.reclaimable
            + self.volumes.reclaimable
            + self.build_cache.reclaimable
    }

    /// Get reclaimable percentage (0-100)
    pub fn reclaimable_percentage(&self) -> f64 {
        let total = self.total_size();
        if total == 0 {
            0.0
        } else {
            (self.total_reclaimable() as f64 / total as f64) * 100.0
        }
    }
}

/// Convert bytes to human-readable format
pub fn format_bytes_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes < 1024 * 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else {
        format!("{:.2} TB", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    }
}

/// Prune options for different resource types
#[derive(Debug, Clone, Default)]
pub struct PruneOptions {
    /// Prune stopped containers
    pub containers: bool,
    /// Prune dangling images
    pub images: bool,
    /// Prune unused volumes
    pub volumes: bool,
    /// Prune unused networks
    pub networks: bool,
    /// Prune build cache (not available in bollard 0.18)
    pub build_cache: bool,
}

/// Result of a prune operation
#[derive(Debug, Clone, Default)]
pub struct PruneResult {
    /// Space reclaimed in bytes
    pub space_reclaimed: i64,
    /// Items that were deleted
    pub items_deleted: Vec<String>,
}

impl PruneResult {
    /// Merge multiple prune results into one
    pub fn merge(results: Vec<PruneResult>) -> Self {
        let mut total = Self::default();
        for result in results {
            total.space_reclaimed += result.space_reclaimed;
            total.items_deleted.extend(result.items_deleted);
        }
        total
    }
}

/// System information from Docker
#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub version: String,
    pub api_version: String,
    pub os: String,
    pub arch: String,
    pub kernel_version: String,
}

use super::client::DockerClient;
use anyhow::Result;

impl DockerClient {
    /// Get system disk usage information
    pub async fn get_disk_usage(&self) -> Result<SystemDiskUsage> {
        let response = self.inner().df().await?;
        Ok(parse_disk_usage(response))
    }

    /// Get system information
    pub async fn get_system_info(&self) -> Result<SystemInfo> {
        let version = self.inner().version().await?;
        let info = self.inner().info().await?;

        Ok(SystemInfo {
            version: version.version.unwrap_or_default(),
            api_version: version.api_version.unwrap_or_default(),
            os: version.os.unwrap_or_default(),
            arch: version.arch.unwrap_or_default(),
            kernel_version: info.kernel_version.unwrap_or_default(),
        })
    }

    /// Prune stopped containers (with detailed result)
    pub async fn prune_containers_detailed(&self) -> Result<PruneResult> {
        let response = self
            .inner()
            .prune_containers(None::<bollard::container::PruneContainersOptions<String>>)
            .await?;
        Ok(PruneResult {
            space_reclaimed: response.space_reclaimed.unwrap_or(0),
            items_deleted: response.containers_deleted.unwrap_or_default(),
        })
    }
}

/// Parse SystemDataUsageResponse into our SystemDiskUsage
fn parse_disk_usage(response: SystemDataUsageResponse) -> SystemDiskUsage {
    let mut usage = SystemDiskUsage::default();

    // Parse images
    if let Some(images) = response.images {
        for image in images {
            usage.images.total += image.size;
            usage.images.count += 1;
            // Dangling images (untagged and not used by any container) are reclaimable
            // Note: containers field can be -1 if not calculated
            let is_unused = image.containers >= 0 && image.containers == 0;
            let is_dangling = image.repo_tags.is_empty();
            if is_unused && is_dangling {
                usage.images.reclaimable += image.size;
            }
        }
    }

    // Parse containers
    if let Some(containers) = response.containers {
        for container in containers {
            let size = container.size_root_fs.unwrap_or(0);
            usage.containers.total += size;
            usage.containers.count += 1;
            // Stopped containers are reclaimable
            if container.state.as_deref() != Some("running") {
                usage.containers.reclaimable += size;
            }
        }
    }

    // Parse volumes
    if let Some(volumes) = response.volumes {
        for volume in volumes {
            // usage_data is only available for local volumes
            let (size, ref_count) = volume
                .usage_data
                .as_ref()
                .map(|u| (u.size, u.ref_count))
                .unwrap_or((-1, -1));
            
            // Only count size if it's available (not -1)
            if size >= 0 {
                usage.volumes.total += size;
            }
            usage.volumes.count += 1;
            
            // Volumes not used by any container (ref_count == 0) are reclaimable
            // Only mark as reclaimable if we know the size
            if ref_count == 0 && size >= 0 {
                usage.volumes.reclaimable += size;
            }
        }
    }

    // Parse build cache
    if let Some(build_cache) = response.build_cache {
        for cache in build_cache {
            let size = cache.size.unwrap_or(0);
            usage.build_cache.total += size;
            usage.build_cache.count += 1;
            // Build cache is reclaimable, but bollard 0.18 doesn't support pruning it
            // Mark as 0 reclaimable since we can't prune it via API
            usage.build_cache.reclaimable += 0;
        }
    }

    usage
}
