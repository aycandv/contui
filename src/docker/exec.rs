//! Exec helper utilities

/// Detect whether a command vector looks like a shell invocation.
pub fn looks_like_shell(cmd: &[String]) -> bool {
    if cmd.is_empty() {
        return false;
    }
    let first = cmd[0].rsplit('/').next().unwrap_or(&cmd[0]);
    matches!(first, "sh" | "bash" | "zsh" | "ash" | "dash")
        || cmd.iter().any(|c| c == "-lc" || c == "-c")
}

/// Select the command to run for exec based on entrypoint/cmd and shell detection.
pub fn select_exec_command(entrypoint: &[String], cmd: &[String]) -> Vec<String> {
    if looks_like_shell(entrypoint) || looks_like_shell(cmd) {
        let mut out = Vec::new();
        out.extend_from_slice(entrypoint);
        out.extend_from_slice(cmd);
        if out.is_empty() {
            return vec!["/bin/sh".to_string(), "-lc".to_string()];
        }
        out
    } else {
        vec!["/bin/sh".to_string(), "-lc".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_shell_entrypoint_with_cmd() {
        let entrypoint = vec!["/bin/sh".to_string(), "-lc".to_string()];
        let cmd = vec!["echo".to_string(), "hi".to_string()];
        let selected = select_exec_command(&entrypoint, &cmd);
        assert_eq!(selected, vec!["/bin/sh", "-lc", "echo", "hi"]);
    }

    #[test]
    fn falls_back_to_sh_when_not_shell() {
        let entrypoint = vec!["/usr/bin/myapp".to_string()];
        let cmd = vec!["--port".to_string(), "8080".to_string()];
        let selected = select_exec_command(&entrypoint, &cmd);
        assert_eq!(selected, vec!["/bin/sh", "-lc"]);
    }

    #[test]
    fn detects_shell_by_basename() {
        assert!(looks_like_shell(&["/bin/bash".to_string()]));
        assert!(!looks_like_shell(&["/usr/bin/python".to_string()]));
    }
}
