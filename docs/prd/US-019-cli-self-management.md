# US-019: CLI Self-Management (Update & Uninstall)

## User Story
As a Contui user, I want to update or uninstall Contui directly from the CLI without using external package managers, so that I can easily manage the tool.

## Acceptance Criteria

### Update Command
- [ ] `contui update` checks for the latest release on GitHub
- [ ] Shows current version and latest version
- [ ] Downloads and installs the latest version if newer
- [ ] Shows progress during download
- [ ] Backs up current binary before updating (optional)
- [ ] Shows success/failure message
- [ ] `contui update --check` only checks without installing

### Uninstall Command
- [ ] `contui uninstall` removes the binary from the system
- [ ] Shows confirmation prompt before uninstalling
- [ ] Removes configuration files optionally (`--purge`)
- [ ] Shows what was removed

### Version Command Enhancement
- [ ] `contui --version` shows current version
- [ ] `contui version` also shows version
- [ ] Shows install source (cargo, install script, etc.) if possible

## Technical Notes

### Self-Update Implementation
- Use `self_update` crate or implement manually with `reqwest`
- Check GitHub releases API for latest version
- Compare semver versions
- Download appropriate binary for current platform
- Replace current binary atomically
- On Windows: may need to use a temporary batch file due to file locking

### Uninstall Implementation
- Detect install location from `std::env::current_exe()`
- Remove the binary
- Optionally remove config directory (~/.config/contui)

## UI/UX

```bash
# Check for updates
$ contui update --check
Current version: 0.1.2
Latest version: 0.2.0
Update available! Run 'contui update' to install.

# Update
$ contui update
Checking for updates...
Found new version: 0.2.0 (current: 0.1.2)
Downloading contui-aarch64-apple-darwin.tar.gz...
[████████████████████] 100%
Installing...
Successfully updated to v0.2.0!

# Already up to date
$ contui update
Checking for updates...
You're already on the latest version (v0.1.2).

# Uninstall
$ contui uninstall
This will remove contui from your system.
Configuration files at ~/.config/contui will NOT be removed.
Are you sure? [y/N] y
Removing /Users/avit/.cargo/bin/contui...
Successfully uninstalled contui.
To remove configuration files: rm -rf ~/.config/contui

# Uninstall with purge
$ contui uninstall --purge
This will remove contui and ALL configuration files.
Are you sure? [y/N] y
Removing /Users/avit/.cargo/bin/contui...
Removing ~/.config/contui...
Successfully uninstalled contui.
```

## Definition of Done
- [ ] `contui update` command works
- [ ] `contui update --check` command works
- [ ] `contui uninstall` command works
- [ ] `contui uninstall --purge` command works
- [ ] All commands have proper error handling
- [ ] Help text updated
