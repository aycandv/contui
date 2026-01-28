use thiserror::Error;

/// Main error type for DockMon
#[derive(Error, Debug)]
pub enum DockMonError {
    /// Docker API errors
    #[error("Docker error: {0}")]
    Docker(#[from] DockerError),

    /// UI errors
    #[error("UI error: {0}")]
    Ui(#[from] UiError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// General errors
    #[error("{0}")]
    Other(String),
}

/// Docker-specific errors
#[derive(Error, Debug)]
pub enum DockerError {
    /// Connection errors
    #[error("Failed to connect to Docker: {0}")]
    Connection(String),

    /// API response errors
    #[error("Docker API error (code {code}): {message}")]
    ApiError { code: u16, message: String },

    /// Resource not found
    #[error("{resource} not found")]
    NotFound { resource: String },

    /// Operation timeout
    #[error("Operation '{operation}' timed out after {duration}s")]
    Timeout { operation: String, duration: u64 },

    /// Permission denied
    #[error("Permission denied accessing Docker")]
    PermissionDenied,

    /// Container errors
    #[error("Container error: {0}")]
    Container(String),

    /// Image errors
    #[error("Image error: {0}")]
    Image(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Volume errors
    #[error("Volume error: {0}")]
    Volume(String),
}

/// UI-related errors
#[derive(Error, Debug)]
pub enum UiError {
    /// Terminal errors
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Rendering errors
    #[error("Rendering error: {0}")]
    Render(String),

    /// Input handling errors
    #[error("Input error: {0}")]
    Input(String),

    /// Layout errors
    #[error("Layout error: {0}")]
    Layout(String),
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Parse errors
    #[error("Failed to parse configuration: {0}")]
    Parse(String),

    /// Validation errors
    #[error("Configuration validation failed: {0}")]
    Validation(String),

    /// File not found
    #[error("Configuration file not found: {0}")]
    NotFound(String),

    /// Environment errors
    #[error("Environment error: {0}")]
    Environment(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, DockMonError>;

impl DockMonError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DockMonError::Docker(DockerError::Connection(_))
                | DockMonError::Docker(DockerError::Timeout { .. })
        )
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            DockMonError::Docker(DockerError::Connection(_)) => {
                "Could not connect to Docker. Please ensure Docker is running.".to_string()
            }
            DockMonError::Docker(DockerError::PermissionDenied) => {
                "Permission denied. Please check your Docker permissions.".to_string()
            }
            DockMonError::Config(ConfigError::NotFound(_)) => {
                "Configuration file not found. Using defaults.".to_string()
            }
            _ => self.to_string(),
        }
    }
}

impl From<toml::de::Error> for DockMonError {
    fn from(err: toml::de::Error) -> Self {
        DockMonError::Config(ConfigError::Parse(err.to_string()))
    }
}

impl From<toml::ser::Error> for DockMonError {
    fn from(err: toml::ser::Error) -> Self {
        DockMonError::Config(ConfigError::Parse(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DockerError::NotFound {
            resource: "container abc123".to_string(),
        };
        assert_eq!(err.to_string(), "container abc123 not found");
    }

    #[test]
    fn test_retryable_errors() {
        let connection_err = DockMonError::Docker(DockerError::Connection(
            "connection refused".to_string(),
        ));
        assert!(connection_err.is_retryable());

        let not_found_err = DockMonError::Docker(DockerError::NotFound {
            resource: "test".to_string(),
        });
        assert!(!not_found_err.is_retryable());
    }

    #[test]
    fn test_user_messages() {
        let conn_err = DockMonError::Docker(DockerError::Connection(
            "test".to_string(),
        ));
        let msg = conn_err.user_message();
        assert!(msg.contains("Docker"));
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let dockmon_err: DockMonError = io_err.into();
        assert!(matches!(dockmon_err, DockMonError::Io(_)));
    }
}
