# Git Workflow for DockMon

This document outlines the git workflow for developing DockMon using the Ralph PRD stories.

## Branching Strategy

### Main Branches
- `main` - Production-ready code
- `develop` - Integration branch for features

### Feature Branches
Each Ralph story gets its own branch following the pattern:
```
ralph/US-XXX-short-description
```

Examples:
- `ralph/US-001-project-setup`
- `ralph/US-006-container-list-widget`
- `ralph/US-011-log-streaming`

## Workflow

### 1. Starting a New Story

```bash
# Ensure you're on latest main
git checkout main
git pull origin main

# Create feature branch for the story
git checkout -b ralph/US-001-project-setup

# Work on the story...
```

### 2. Commit Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, missing semi colons, etc)
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Build process or auxiliary tool changes

Examples:
```bash
feat(docker): add container list operations

Implements list_containers(), start_container(), stop_container()
as specified in US-004.

Relates to: US-004
```

```bash
docs(architecture): add data flow diagrams

Add sequence diagrams for container refresh and log search flows.

Relates to: US-001
```

### 3. Story Completion Checklist

Before committing:
- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy warnings resolved (`cargo clippy`)
- [ ] Commit message follows convention

```bash
# Run checks
cargo check
cargo test
cargo fmt --check
cargo clippy -- -D warnings

# Stage and commit
git add .
git commit -m "feat(containers): implement basic container operations

Add container list view with start/stop/restart/pause/kill/remove
operations as specified in US-004 and US-008.

- Implement list_containers() with auto-refresh
- Add keyboard shortcuts (s, r, p, k, d)
- Add confirmation dialogs for destructive actions
- Show notifications on action completion

Relates to: US-004, US-008"
```

### 4. Update Ralph Progress

After completing a story:

```bash
# Update prd.json to mark story as complete
# Edit ralph/prd.json, set "passes": true for the story

# Update progress.txt (if using Ralph tracking)
echo "US-XXX completed: <brief description>" >> ralph/progress.txt

git add ralph/
git commit -m "docs(ralph): mark US-XXX as complete"
```

### 5. Merge Strategy

```bash
# When story is complete
git checkout main
git pull origin main
git merge --no-ff ralph/US-XXX-short-description

# Push to remote
git push origin main

# Delete feature branch
git branch -d ralph/US-XXX-short-description
```

## Commit History Example

```
* feat(ui): add container detail panel (US-007)
* feat(ui): implement container list widget (US-006)
* feat(docker): add container list and operations (US-004)
* feat(core): add Docker client wrapper (US-003)
* feat(core): define core types and errors (US-002)
* chore(project): initial project setup (US-001)
```

## Ralph Integration

### Using Ralph with Git

1. **Ralph creates a branch** for each story automatically:
   ```bash
   ralph.sh US-001
   # Creates: ralph/US-001-project-setup
   ```

2. **Track progress** in `ralph/progress.txt`:
   ```
   === Ralph Progress ===
   Started: 2024-01-28
   
   [X] US-001: Project setup and dependencies
   [X] US-002: Core types and error handling
   [ ] US-003: Docker client wrapper and connection
   ```

3. **Archive completed stories**:
   ```bash
   mkdir -p ralph/archive/$(date +%Y-%m-%d)
   cp ralph/prd.json ralph/archive/$(date +%Y-%m-%d)/
   cp ralph/progress.txt ralph/archive/$(date +%Y-%m-%d)/
   ```

## Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Check formatting
echo "Checking formatting..."
cargo fmt -- --check

# Run clippy
echo "Running clippy..."
cargo clippy -- -D warnings

# Run tests
echo "Running tests..."
cargo test

echo "All checks passed!"
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

## Release Workflow

### Version Tagging

```bash
# After US-025 (Release packaging)
git checkout main
git tag -a v0.1.0 -m "Initial MVP release

Features:
- Container list and lifecycle management
- Image, volume, network management
- Real-time logs and stats
- Docker Compose support
- Customizable keybindings and themes

Closes: US-001 through US-025"

git push origin v0.1.0
```

### Release Checklist

- [ ] All US-XXX stories complete in Ralph PRD
- [ ] Version bumped in `Cargo.toml`
- [ ] CHANGELOG.md updated
- [ ] Git tag created
- [ ] GitHub release created with binaries

## Git Best Practices

### Atomic Commits
Each commit should represent a single logical change:
- ✅ Good: "feat(docker): add container pause/unpause"
- ❌ Bad: "updates" or "various fixes"

### Commit Frequency
Commit early and often:
- After completing a function
- After fixing a bug
- Before switching contexts

### Branch Hygiene
- Keep branches focused on single story
- Delete merged branches
- Rebase feature branches on main before merge

### Documentation in Commits
- Explain WHY, not just WHAT
- Reference related stories: "Relates to: US-004"
- Mention breaking changes in footer

## Troubleshooting

### Large Files
```bash
# Check for large files
git ls-files | xargs -I{} sh -c 'du -h {}' | sort -hr | head -20

# Add to .gitignore if needed
echo "*.large" >> .gitignore
```

### Merge Conflicts
```bash
# When conflict occurs
git status
# Edit conflicting files
git add <resolved-files>
git commit -m "merge: resolve conflicts in US-XXX"
```

### Recovering Work
```bash
# See recent reflog
git reflog

# Recover lost commit
git checkout -b recovery-branch HEAD@{5}
```
