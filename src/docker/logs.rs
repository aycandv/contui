//! Log streaming operations

use bollard::container::LogsOptions;
use futures::StreamExt;
use tracing::debug;

use crate::core::{DockerError, Result};
use crate::docker::DockerClient;

/// Log entry from a container
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub message: String,
    pub is_stderr: bool,
}

impl DockerClient {
    /// Stream logs from a container
    pub fn stream_logs(
        &self,
        id: &str,
        follow: bool,
        tail: usize,
    ) -> impl futures::Stream<Item = Result<LogEntry>> + '_ {
        use crate::core::DockMonError;

        let options = LogsOptions {
            stdout: true,
            stderr: true,
            timestamps: true,
            follow,
            tail: tail.to_string(),
            ..Default::default()
        };

        let stream = self.inner().logs(id, Some(options));

        stream.map(|result| {
            result
                .map_err(|e| DockMonError::Docker(DockerError::Container(format!("Failed to read logs: {}", e))))
                .and_then(|log| Self::parse_log_entry(log))
        })
    }

    /// Parse a log entry from bollard
    fn parse_log_entry(log: bollard::container::LogOutput) -> crate::core::Result<LogEntry> {
        use crate::core::{DockMonError, DockerError};

        // Extract message from log output
        let (message, is_stderr) = match log {
            bollard::container::LogOutput::StdOut { message } => {
                (String::from_utf8_lossy(&message).to_string(), false)
            }
            bollard::container::LogOutput::StdErr { message } => {
                (String::from_utf8_lossy(&message).to_string(), true)
            }
            _ => {
                return Err(DockMonError::Docker(DockerError::Container("Unknown log output type".to_string())));
            }
        };

        // Parse timestamp from message (format: "2024-01-28T10:30:00.123456789Z message")
        let (timestamp, message) = if let Some(pos) = message.find(' ') {
            let ts_str = &message[..pos];
            let msg = message[pos + 1..].to_string();
            let timestamp = chrono::DateTime::parse_from_rfc3339(ts_str)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc));
            (timestamp, msg)
        } else {
            (None, message)
        };

        Ok(LogEntry {
            timestamp,
            message,
            is_stderr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require Docker to be running

    #[tokio::test]
    #[ignore = "requires Docker daemon"]
    async fn test_stream_logs() {
        let client = DockerClient::from_env().await.unwrap();
        // This would need a running container to test properly
    }
}
