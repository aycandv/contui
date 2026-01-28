//! Application state management

use chrono::Utc;

use crate::core::{
    ConfirmAction, ConnectionInfo, ContainerSummary, ImageSummary, NetworkSummary,
    NotificationLevel, Tab, VolumeSummary,
};

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    // Navigation
    pub current_tab: Tab,
    pub previous_tab: Option<Tab>,
    pub focused_panel: Panel,

    // Docker data
    pub containers: Vec<ContainerSummary>,
    pub selected_container: Option<String>,
    pub container_list_selected: usize,
    pub images: Vec<ImageSummary>,
    pub selected_image: Option<String>,
    pub image_list_selected: usize,
    pub volumes: Vec<VolumeSummary>,
    pub networks: Vec<NetworkSummary>,

    // Connection
    pub docker_connected: bool,
    pub connection_info: ConnectionInfo,

    // UI state
    pub terminal_size: (u16, u16),
    pub show_help: bool,
    pub notifications: Vec<Notification>,
    pub confirm_dialog: Option<ConfirmAction>,

    // Async operations tracking
    pub loading: bool,
}

/// Panel focus areas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Sidebar,
    Main,
}

/// Notification message
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: uuid::Uuid,
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AppState {
    /// Create new app state
    pub fn new() -> Self {
        Self {
            current_tab: Tab::Containers,
            previous_tab: None,
            focused_panel: Panel::Sidebar,
            containers: vec![],
            selected_container: None,
            container_list_selected: 0,
            images: vec![],
            selected_image: None,
            image_list_selected: 0,
            volumes: vec![],
            networks: vec![],
            docker_connected: false,
            connection_info: ConnectionInfo::default(),
            terminal_size: (80, 24),
            show_help: false,
            notifications: vec![],
            confirm_dialog: None,
            loading: false,
        }
    }

    /// Add a notification
    pub fn add_notification(&mut self, message: impl Into<String>, level: NotificationLevel) {
        let notification = Notification {
            id: uuid::Uuid::new_v4(),
            message: message.into(),
            level,
            timestamp: Utc::now(),
        };
        self.notifications.push(notification);

        // Keep only last 10 notifications
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    /// Clear old notifications (older than threshold)
    pub fn clear_old_notifications(&mut self, max_age_seconds: i64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_seconds);
        self.notifications.retain(|n| n.timestamp > cutoff);
    }

    /// Update images list
    pub fn update_images(&mut self, images: Vec<ImageSummary>) {
        self.images = images;
        // Adjust selection if needed
        if !self.images.is_empty() {
            if self.image_list_selected >= self.images.len() {
                self.image_list_selected = self.images.len() - 1;
            }
            self.selected_image = Some(
                self.images[self.image_list_selected].id.clone()
            );
        } else {
            self.image_list_selected = 0;
            self.selected_image = None;
        }
    }

    /// Navigate to next image in list
    pub fn next_image(&mut self) {
        if self.images.is_empty() {
            return;
        }
        self.image_list_selected = 
            (self.image_list_selected + 1) % self.images.len();
        self.selected_image = Some(
            self.images[self.image_list_selected].id.clone()
        );
    }

    /// Navigate to previous image in list
    pub fn previous_image(&mut self) {
        if self.images.is_empty() {
            return;
        }
        if self.image_list_selected == 0 {
            self.image_list_selected = self.images.len() - 1;
        } else {
            self.image_list_selected -= 1;
        }
        self.selected_image = Some(
            self.images[self.image_list_selected].id.clone()
        );
    }

    /// Update containers list
    pub fn update_containers(&mut self, containers: Vec<ContainerSummary>) {
        self.containers = containers;
        // Adjust selection if needed
        if !self.containers.is_empty() {
            if self.container_list_selected >= self.containers.len() {
                self.container_list_selected = self.containers.len() - 1;
            }
            self.selected_container = Some(
                self.containers[self.container_list_selected].id.clone()
            );
        } else {
            self.container_list_selected = 0;
            self.selected_container = None;
        }
    }

    /// Navigate to next container in list
    pub fn next_container(&mut self) {
        if self.containers.is_empty() {
            return;
        }
        self.container_list_selected = 
            (self.container_list_selected + 1) % self.containers.len();
        self.selected_container = Some(
            self.containers[self.container_list_selected].id.clone()
        );
    }

    /// Navigate to previous container in list
    pub fn previous_container(&mut self) {
        if self.containers.is_empty() {
            return;
        }
        if self.container_list_selected == 0 {
            self.container_list_selected = self.containers.len() - 1;
        } else {
            self.container_list_selected -= 1;
        }
        self.selected_container = Some(
            self.containers[self.container_list_selected].id.clone()
        );
    }

    /// Update volumes list
    pub fn update_volumes(&mut self, volumes: Vec<VolumeSummary>) {
        self.volumes = volumes;
    }

    /// Update networks list
    pub fn update_networks(&mut self, networks: Vec<NetworkSummary>) {
        self.networks = networks;
    }

    /// Set Docker connection status
    pub fn set_docker_connected(&mut self, connected: bool, info: ConnectionInfo) {
        self.docker_connected = connected;
        self.connection_info = info;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.current_tab, Tab::Containers);
        assert!(state.containers.is_empty());
        assert!(!state.docker_connected);
    }

    #[test]
    fn test_add_notification() {
        let mut state = AppState::default();
        state.add_notification("Test message", NotificationLevel::Info);

        assert_eq!(state.notifications.len(), 1);
        assert_eq!(state.notifications[0].message, "Test message");
    }

    #[test]
    fn test_notification_limit() {
        let mut state = AppState::default();

        // Add 15 notifications
        for i in 0..15 {
            state.add_notification(format!("Message {}", i), NotificationLevel::Info);
        }

        // Should only keep last 10
        assert_eq!(state.notifications.len(), 10);
    }

    #[test]
    fn test_update_containers() {
        let mut state = AppState::default();
        let containers = vec![ContainerSummary::default()];

        state.update_containers(containers);
        assert_eq!(state.containers.len(), 1);
    }
}
