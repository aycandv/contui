//! Core type definitions and shared types

/// Type alias for container IDs
pub type ContainerId = String;

/// Type alias for image IDs
pub type ImageId = String;

/// Type alias for volume names
pub type VolumeName = String;

/// Type alias for network IDs
pub type NetworkId = String;

/// Type alias for operation IDs
pub type OperationId = uuid::Uuid;

use uuid::Uuid;

/// Generate a new unique operation ID
pub fn new_operation_id() -> OperationId {
    Uuid::new_v4()
}

/// Notification level for status messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl std::fmt::Display for NotificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationLevel::Info => write!(f, "INFO"),
            NotificationLevel::Success => write!(f, "SUCCESS"),
            NotificationLevel::Warning => write!(f, "WARNING"),
            NotificationLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Filter operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    Contains,
    Equals,
    Regex,
    GreaterThan,
    LessThan,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// Toggle the sort direction
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// Application tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tab {
    Containers,
    Images,
    Volumes,
    Networks,
    Compose,
    System,
}

impl Tab {
    /// Get all available tabs
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Containers,
            Tab::Images,
            Tab::Volumes,
            Tab::Networks,
            Tab::Compose,
            Tab::System,
        ]
    }

    /// Get the display name for this tab
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Containers => "Containers",
            Tab::Images => "Images",
            Tab::Volumes => "Volumes",
            Tab::Networks => "Networks",
            Tab::Compose => "Compose",
            Tab::System => "System",
        }
    }

    /// Get the shortcut key for this tab (1-6)
    pub fn shortcut(&self) -> char {
        match self {
            Tab::Containers => '1',
            Tab::Images => '2',
            Tab::Volumes => '3',
            Tab::Networks => '4',
            Tab::Compose => '5',
            Tab::System => '6',
        }
    }
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Panel focus areas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    List,
    Detail,
    Logs,
    Stats,
}

/// Modal dialog types
#[derive(Debug, Clone)]
pub enum Modal {
    Confirm(ConfirmDialog),
    Input(InputDialog),
    Help(HelpContent),
    Error(String),
}

/// Confirmation dialog content
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub cancel_label: String,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
        }
    }

    pub fn with_labels(
        mut self,
        confirm: impl Into<String>,
        cancel: impl Into<String>,
    ) -> Self {
        self.confirm_label = confirm.into();
        self.cancel_label = cancel.into();
        self
    }
}

/// Input dialog content
#[derive(Debug, Clone)]
pub struct InputDialog {
    pub title: String,
    pub prompt: String,
    pub default_value: Option<String>,
    pub placeholder: String,
}

/// Help content
#[derive(Debug, Clone)]
pub struct HelpContent {
    pub title: String,
    pub sections: Vec<HelpSection>,
}

/// Help section
#[derive(Debug, Clone)]
pub struct HelpSection {
    pub title: String,
    pub items: Vec<(String, String)>, // (key, description)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_direction_toggle() {
        let asc = SortDirection::Ascending;
        assert_eq!(asc.toggle(), SortDirection::Descending);
        assert_eq!(asc.toggle().toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_tab_properties() {
        assert_eq!(Tab::Containers.name(), "Containers");
        assert_eq!(Tab::Containers.shortcut(), '1');
        assert_eq!(Tab::all().len(), 6);
    }

    #[test]
    fn test_notification_level_display() {
        assert_eq!(NotificationLevel::Error.to_string(), "ERROR");
        assert_eq!(NotificationLevel::Success.to_string(), "SUCCESS");
    }

    #[test]
    fn test_confirm_dialog_builder() {
        let dialog = ConfirmDialog::new("Title", "Message")
            .with_labels("Yes", "No");
        
        assert_eq!(dialog.title, "Title");
        assert_eq!(dialog.confirm_label, "Yes");
        assert_eq!(dialog.cancel_label, "No");
    }
}
