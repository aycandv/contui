//! Docker container inspection

use crate::core::{DockMonError, DockerError, Result};
use crate::docker::DockerClient;

/// Container details from inspect
#[derive(Debug, Clone)]
pub struct ContainerDetails {
    pub id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub status: String,
    pub state: ContainerState,
    pub created: String,
    pub restart_policy: String,
    pub command: Option<String>,
    pub entrypoint: Option<Vec<String>>,
    pub ports: Vec<PortMapping>,
    pub mounts: Vec<Mount>,
    pub env: Vec<String>,
    pub labels: Vec<(String, String)>,
    pub networks: Vec<NetworkInfo>,
}

#[derive(Debug, Clone)]
pub struct ContainerState {
    pub running: bool,
    pub paused: bool,
    pub restarting: bool,
    pub exit_code: i64,
    pub error: String,
    pub health: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PortMapping {
    pub port: u16,
    pub protocol: String,
    pub host_ip: String,
    pub host_port: u16,
}

#[derive(Debug, Clone)]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub mount_type: String,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub ip_address: String,
    pub gateway: String,
    pub mac_address: String,
}

impl DockerClient {
    /// Inspect a container and get detailed information
    pub async fn inspect_container(&self, id: &str) -> Result<ContainerDetails> {
        use tracing::debug;

        debug!("Inspecting container {}", id);

        let inspect = self
            .inner()
            .inspect_container(id, None)
            .await
            .map_err(|e| {
                DockMonError::Docker(DockerError::Container(format!("Failed to inspect: {}", e)))
            })?;

        // Extract state
        let state = if let Some(s) = &inspect.state {
            ContainerState {
                running: s.running.unwrap_or(false),
                paused: s.paused.unwrap_or(false),
                restarting: s.restarting.unwrap_or(false),
                exit_code: s.exit_code.unwrap_or(0),
                error: s.error.clone().unwrap_or_default(),
                health: s.health.as_ref().map(|h| format!("{:?}", h.status)),
                started_at: s.started_at.clone(),
                finished_at: s.finished_at.clone(),
            }
        } else {
            ContainerState {
                running: false,
                paused: false,
                restarting: false,
                exit_code: 0,
                error: String::new(),
                health: None,
                started_at: None,
                finished_at: None,
            }
        };

        // Store host_config reference for reuse
        let host_config_ref = inspect.host_config.as_ref();

        // Extract ports
        let mut ports = Vec::new();
        if let Some(port_bindings) = host_config_ref.and_then(|hc| hc.port_bindings.as_ref()) {
            for (container_port, host_bindings) in port_bindings {
                // Parse "8080/tcp" -> (8080, "tcp")
                let parts: Vec<&str> = container_port.split('/').collect();
                let port_num = parts[0].parse::<u16>().unwrap_or(0);
                let protocol = parts.get(1).unwrap_or(&"tcp").to_string();

                if let Some(bindings) = host_bindings {
                    for binding in bindings {
                        ports.push(PortMapping {
                            port: port_num,
                            protocol: protocol.clone(),
                            host_ip: binding
                                .host_ip
                                .clone()
                                .unwrap_or_else(|| "0.0.0.0".to_string()),
                            host_port: binding
                                .host_port
                                .clone()
                                .unwrap_or_default()
                                .parse::<u16>()
                                .unwrap_or(0),
                        });
                    }
                }
            }
        }

        // Extract mounts
        let mounts = inspect
            .mounts
            .map(|m| {
                m.into_iter()
                    .map(|mount| Mount {
                        source: mount.source.unwrap_or_default(),
                        destination: mount.destination.unwrap_or_default(),
                        mode: mount.mode.unwrap_or_else(|| "rw".to_string()),
                        mount_type: mount
                            .typ
                            .map(|t| format!("{:?}", t))
                            .unwrap_or_else(|| "bind".to_string()),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract env vars
        let env = inspect
            .config
            .as_ref()
            .and_then(|c| c.env.clone())
            .unwrap_or_default();

        // Extract labels
        let labels = inspect
            .config
            .as_ref()
            .and_then(|c| c.labels.clone())
            .map(|l| l.into_iter().collect::<Vec<_>>())
            .unwrap_or_default();

        // Extract network info
        let mut networks = Vec::new();
        if let Some(network_settings) = &inspect.network_settings {
            if let Some(networks_map) = &network_settings.networks {
                for (name, network) in networks_map {
                    networks.push(NetworkInfo {
                        name: name.clone(),
                        ip_address: network.ip_address.clone().unwrap_or_default(),
                        gateway: network.gateway.clone().unwrap_or_default(),
                        mac_address: network.mac_address.clone().unwrap_or_default(),
                    });
                }
            }
        }

        // Extract restart policy
        let restart_policy = host_config_ref
            .and_then(|hc| hc.restart_policy.as_ref())
            .and_then(|rp| rp.name.as_ref())
            .map(|p| format!("{:?}", p))
            .unwrap_or_else(|| "no".to_string());

        // Extract command and entrypoint
        let command = inspect
            .config
            .as_ref()
            .and_then(|c| c.cmd.clone())
            .map(|cmd| cmd.join(" "));

        let entrypoint = inspect.config.as_ref().and_then(|c| c.entrypoint.clone());

        Ok(ContainerDetails {
            id: inspect.id.unwrap_or_default().chars().take(12).collect(),
            name: inspect
                .name
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string(),
            image: inspect
                .config
                .as_ref()
                .and_then(|c| c.image.clone())
                .unwrap_or_default(),
            image_id: inspect.image.unwrap_or_default().chars().take(12).collect(),
            status: format_status(&state),
            state,
            created: inspect.created.unwrap_or_default(),
            restart_policy,
            command,
            entrypoint,
            ports,
            mounts,
            env,
            labels,
            networks,
        })
    }
}

/// Format container status string
fn format_status(state: &ContainerState) -> String {
    if state.restarting {
        "Restarting".to_string()
    } else if state.running {
        state
            .health
            .as_ref()
            .map(|h| format!("Running ({})", h.to_lowercase()))
            .unwrap_or_else(|| "Running".to_string())
    } else if state.paused {
        "Paused".to_string()
    } else if state.exit_code != 0 {
        format!("Exited ({})", state.exit_code)
    } else {
        "Exited (0)".to_string()
    }
}
